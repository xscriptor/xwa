"""
SEO Analysis Module.

Extracts meta tags, social tags, headings, image alts, text ratio,
canonical URL, viewport, structured data, link relationships, and robots.txt.
"""

from bs4 import BeautifulSoup
from typing import Dict, Optional, List, Any
import json
import re


def extract_standard_meta_tags(html_content: str) -> Dict[str, Optional[str]]:
    """Extracts standard SEO meta tags including viewport and charset."""
    soup = BeautifulSoup(html_content, 'html.parser')

    results: Dict[str, Optional[str]] = {
        'title': None,
        'description': None,
        'keywords': None,
        'author': None,
        'robots': None,
        'viewport': None,
        'charset': None,
        'generator': None,
        'theme_color': None,
    }

    # Title
    title_tag = soup.find('title')
    if title_tag:
        results['title'] = title_tag.get_text(strip=True)

    # Charset
    charset_meta = soup.find('meta', attrs={'charset': True})
    if charset_meta:
        results['charset'] = charset_meta.get('charset', '').upper()
    else:
        ct_meta = soup.find('meta', attrs={'http-equiv': lambda x: x and x.lower() == 'content-type'})
        if ct_meta:
            content = ct_meta.get('content', '')
            if 'charset=' in content.lower():
                results['charset'] = content.lower().split('charset=')[1].strip().upper()

    # Standard meta by name
    for meta in soup.find_all('meta'):
        name = (meta.get('name') or '').lower()
        content = meta.get('content', '')

        if name and content:
            if name in ['description', 'keywords', 'author', 'robots', 'generator']:
                results[name] = content.strip()
            elif name == 'viewport':
                results['viewport'] = content.strip()
            elif name == 'theme-color':
                results['theme_color'] = content.strip()

    return results


def extract_social_meta_tags(html_content: str) -> Dict[str, Dict[str, str]]:
    """Extracts Open Graph and Twitter Card meta tags.
    Handles both property= and name= attributes for OG tags.
    """
    soup = BeautifulSoup(html_content, 'html.parser')
    results: Dict[str, Dict[str, str]] = {'og': {}, 'twitter': {}}

    for meta in soup.find_all('meta'):
        property_attr = meta.get('property', '') or ''
        name_attr = meta.get('name', '') or ''
        content = meta.get('content', '')

        if not content:
            continue

        # OG can be in property= or name= attribute
        og_key = ''
        if property_attr.lower().startswith('og:'):
            og_key = property_attr.split(':', 1)[1] if ':' in property_attr else ''
        elif name_attr.lower().startswith('og:'):
            og_key = name_attr.split(':', 1)[1] if ':' in name_attr else ''

        if og_key:
            results['og'][og_key.lower()] = content.strip()
            continue

        # Twitter can be in name= or property=
        tw_key = ''
        if name_attr.lower().startswith('twitter:'):
            tw_key = name_attr.split(':', 1)[1] if ':' in name_attr else ''
        elif property_attr.lower().startswith('twitter:'):
            tw_key = property_attr.split(':', 1)[1] if ':' in property_attr else ''

        if tw_key:
            results['twitter'][tw_key.lower()] = content.strip()

    return results


def extract_structured_data(html_content: str) -> Dict[str, Any]:
    """Extracts JSON-LD structured data and microdata."""
    soup = BeautifulSoup(html_content, 'html.parser')
    results: Dict[str, Any] = {
        'json_ld': [],
        'microdata_types': [],
    }

    # JSON-LD
    for script in soup.find_all('script', attrs={'type': 'application/ld+json'}):
        try:
            data = json.loads(script.string or '{}')
            # Extract summary
            if isinstance(data, dict):
                results['json_ld'].append({
                    'type': data.get('@type', 'Unknown'),
                    'name': data.get('name', data.get('headline', '')),
                    'url': data.get('url', ''),
                    'raw_keys': list(data.keys())[:15]
                })
            elif isinstance(data, list):
                for item in data[:5]:
                    if isinstance(item, dict):
                        results['json_ld'].append({
                            'type': item.get('@type', 'Unknown'),
                            'name': item.get('name', item.get('headline', '')),
                            'raw_keys': list(item.keys())[:15]
                        })
        except (json.JSONDecodeError, TypeError):
            pass

    # Microdata (itemscope)
    for el in soup.find_all(attrs={'itemscope': True}):
        itemtype = el.get('itemtype', '')
        if itemtype:
            results['microdata_types'].append(itemtype)

    return results


def extract_link_tags(html_content: str) -> List[Dict[str, str]]:
    """Extracts important <link> tags (alternate, prev, next, icon, etc.)."""
    soup = BeautifulSoup(html_content, 'html.parser')
    results = []

    important_rels = ['canonical', 'alternate', 'prev', 'next', 'icon', 'apple-touch-icon',
                      'manifest', 'stylesheet', 'preconnect', 'dns-prefetch', 'preload']

    for link in soup.find_all('link'):
        rel = link.get('rel', [])
        if isinstance(rel, list):
            rel_str = ' '.join(rel).lower()
        else:
            rel_str = str(rel).lower()

        for important in important_rels:
            if important in rel_str:
                results.append({
                    'rel': rel_str,
                    'href': link.get('href', '')[:200],
                    'type': link.get('type', ''),
                    'hreflang': link.get('hreflang', ''),
                })
                break

    return results


def analyze_headings(html_content: str) -> Dict[str, Any]:
    """Heading metrics with text content and nesting detail."""
    soup = BeautifulSoup(html_content, 'html.parser')
    counts = {'h1': 0, 'h2': 0, 'h3': 0, 'h4': 0, 'h5': 0, 'h6': 0}
    details = []

    for h in ['h1', 'h2', 'h3', 'h4', 'h5', 'h6']:
        tags = soup.find_all(h)
        counts[h] = len(tags)
        for tag in tags:
            details.append({
                'tag': h.upper(),
                'text': tag.get_text(strip=True)[:120],
                'id': tag.get('id'),
            })

    return {
        'counts': counts,
        'details': details,
        'missing_h1': counts['h1'] == 0,
        'multiple_h1': counts['h1'] > 1
    }


def analyze_image_alts(html_content: str) -> Dict[str, Any]:
    """Image alt attribute audit."""
    soup = BeautifulSoup(html_content, 'html.parser')
    images = soup.find_all('img')

    total = len(images)
    missing_alt = 0
    missing_alt_urls = []

    for img in images:
        alt = img.get('alt')
        if alt is None or alt.strip() == '':
            missing_alt += 1
            src = img.get('src', 'unknown-source')
            missing_alt_urls.append(src[:200])

    return {
        'total_images': total,
        'missing_alt': missing_alt,
        'missing_alt_urls': missing_alt_urls
    }


def analyze_text_ratio(html_content: str) -> Dict[str, Any]:
    """Text-to-HTML ratio and word count."""
    soup = BeautifulSoup(html_content, 'html.parser')

    for script in soup(['script', 'style', 'noscript']):
        script.extract()

    text = soup.get_text()
    lines = (line.strip() for line in text.splitlines())
    chunks = (phrase.strip() for line in lines for phrase in line.split("  "))
    text_content = ' '.join(chunk for chunk in chunks if chunk)

    html_size = len(html_content)
    text_size = len(text_content)

    ratio = (text_size / html_size) * 100 if html_size > 0 else 0
    words = len(text_content.split())

    return {
        'html_size_bytes': html_size,
        'text_size_bytes': text_size,
        'text_to_html_ratio': round(ratio, 2),
        'word_count': words
    }


def extract_canonical(html_content: str) -> Optional[str]:
    """Extracts the canonical URL."""
    soup = BeautifulSoup(html_content, 'html.parser')
    tag = soup.find('link', rel='canonical')
    if tag and tag.get('href'):
        return tag.get('href', '').strip()
    return None


def check_robots_txt(base_url: str) -> Dict[str, Any]:
    """Checks robots.txt presence and extracts directives."""
    from urllib.parse import urlparse
    import requests

    parsed = urlparse(base_url)
    robots_url = f"{parsed.scheme}://{parsed.netloc}/robots.txt"

    try:
        response = requests.get(robots_url, timeout=5)
        exists = response.status_code == 200

        directives = []
        sitemaps = []
        if exists:
            for line in response.text.splitlines():
                line = line.strip()
                if line and not line.startswith('#'):
                    directives.append(line)
                    if line.lower().startswith('sitemap:'):
                        sitemaps.append(line.split(':', 1)[1].strip())

        return {
            'presence': exists,
            'url': robots_url,
            'status_code': response.status_code,
            'directives': directives[:30],
            'sitemaps': sitemaps,
        }
    except Exception:
        return {
            'presence': False,
            'url': robots_url,
            'status_code': 0,
            'directives': [],
            'sitemaps': [],
        }
