"""
HTML Structure Analysis Module.

Extracts the DOM tag tree with IDs, classes, ARIA attributes, roles,
and clean text content. Highlights bad practices.
"""

from bs4 import BeautifulSoup, Tag, NavigableString
from typing import Dict, List, Any, Optional


# Tags that should have accessible labels
INTERACTIVE_TAGS = {'button', 'a', 'input', 'select', 'textarea'}
# Tags that benefit from having an ID
ID_RECOMMENDED_TAGS = {'nav', 'main', 'section', 'article', 'aside', 'header', 'footer', 'form'}
# Semantic landmark tags
LANDMARK_TAGS = {'nav', 'main', 'header', 'footer', 'aside', 'section', 'article'}
# Tags to skip in structure tree
SKIP_TAGS = {'script', 'style', 'noscript', 'svg', 'path', 'link', 'br', 'hr', 'wbr'}


def _get_element_issues(el: Tag) -> List[Dict[str, str]]:
    """Check a single element for accessibility bad practices."""
    issues = []
    tag = el.name

    # Interactive elements need labels
    if tag in INTERACTIVE_TAGS:
        has_label = bool(
            el.get('aria-label') or
            el.get('aria-labelledby') or
            el.get('title')
        )
        if tag in ('button', 'a'):
            text = el.get_text(strip=True)
            if text:
                has_label = True
        if tag == 'input' and el.get('type') == 'hidden':
            has_label = True  # skip hidden
        if tag in ('input', 'select', 'textarea'):
            el_id = el.get('id')
            if el_id:
                # Check if parent soup has label for this id
                root = el.find_parent(['html', '[document]']) or el
                if root.find('label', attrs={'for': el_id}):
                    has_label = True
            if el.find_parent('label'):
                has_label = True
            if el.get('placeholder'):
                has_label = True  # weak but counts

        if not has_label:
            issues.append({
                "type": "missing_label",
                "message": f"<{tag}> missing accessible label"
            })

    # Semantic tags should have IDs for navigation
    if tag in ID_RECOMMENDED_TAGS and not el.get('id'):
        issues.append({
            "type": "missing_id",
            "message": f"<{tag}> has no id attribute (recommended for landmarks)"
        })

    # Images need alt
    if tag == 'img':
        alt = el.get('alt')
        if alt is None:
            issues.append({"type": "missing_alt", "message": "img missing alt attribute"})
        elif alt.strip() == '':
            issues.append({"type": "empty_alt", "message": "img has empty alt attribute"})

    # Role attribute check for generic divs/spans used as interactive
    if tag in ('div', 'span'):
        onclick = el.get('onclick') or el.get('tabindex')
        role = el.get('role')
        if onclick and not role:
            issues.append({
                "type": "missing_role",
                "message": f"<{tag}> has onclick/tabindex but no role attribute"
            })

    return issues


def _build_element_node(el: Tag, depth: int, max_depth: int = 6) -> Optional[Dict[str, Any]]:
    """Recursively build a structure node for a DOM element."""
    if not isinstance(el, Tag):
        return None
    if el.name in SKIP_TAGS:
        return None
    if depth > max_depth:
        return None

    # Gather attributes
    attrs = {}
    if el.get('id'):
        attrs['id'] = el['id']
    if el.get('class'):
        attrs['class'] = ' '.join(el['class']) if isinstance(el['class'], list) else el['class']
    if el.get('role'):
        attrs['role'] = el['role']

    # Gather all aria-* attributes
    aria = {}
    for attr_name, attr_val in el.attrs.items():
        if attr_name.startswith('aria-'):
            aria[attr_name] = attr_val if isinstance(attr_val, str) else ' '.join(attr_val)
    if aria:
        attrs['aria'] = aria

    # Get direct text content (not children's text)
    direct_text_parts = []
    for child in el.children:
        if isinstance(child, NavigableString) and not isinstance(child, Tag):
            t = child.strip()
            if t:
                direct_text_parts.append(t[:100])
    direct_text = ' '.join(direct_text_parts)[:200] if direct_text_parts else None

    # Issues
    issues = _get_element_issues(el)

    # Children
    children = []
    for child in el.children:
        if isinstance(child, Tag):
            child_node = _build_element_node(child, depth + 1, max_depth)
            if child_node:
                children.append(child_node)

    # Some elements like meta-only head are not interesting
    if el.name == 'head':
        return None

    node: Dict[str, Any] = {
        "tag": el.name,
    }
    if attrs:
        node["attrs"] = attrs
    if direct_text:
        node["text"] = direct_text
    if issues:
        node["issues"] = issues
    if children:
        node["children"] = children

    return node


def extract_clean_text(html_content: str) -> Dict[str, Any]:
    """Extract clean, readable text content from HTML."""
    soup = BeautifulSoup(html_content, 'html.parser')

    # Remove script, style, noscript
    for tag in soup(['script', 'style', 'noscript']):
        tag.extract()

    # Extract text by semantic sections
    sections = []

    # Try to get main content areas
    main = soup.find('main') or soup.find('body') or soup
    for el in main.children:
        if isinstance(el, Tag) and el.name not in SKIP_TAGS:
            text = el.get_text(separator=' ', strip=True)
            if text and len(text) > 10:
                sections.append({
                    "tag": el.name,
                    "id": el.get('id'),
                    "text": text[:500]
                })

    # Full clean text
    full_text = soup.get_text(separator=' ', strip=True)
    words = full_text.split()

    return {
        "total_chars": len(full_text),
        "total_words": len(words),
        "sections": sections[:30],  # limit
        "preview": full_text[:1000]
    }


def extract_all_ids(soup: BeautifulSoup) -> List[Dict[str, str]]:
    """Extract all elements with id attributes."""
    results = []
    for el in soup.find_all(attrs={"id": True}):
        if isinstance(el, Tag) and el.name not in SKIP_TAGS:
            results.append({
                "tag": el.name,
                "id": el['id'],
                "text": el.get_text(strip=True)[:80]
            })
    return results


def extract_all_aria(soup: BeautifulSoup) -> List[Dict[str, Any]]:
    """Extract all elements with ARIA attributes."""
    results = []
    for el in soup.find_all():
        if not isinstance(el, Tag) or el.name in SKIP_TAGS:
            continue
        aria_attrs = {k: v for k, v in el.attrs.items() if k.startswith('aria-') or k == 'role'}
        if aria_attrs:
            results.append({
                "tag": el.name,
                "id": el.get('id'),
                "aria": {k: (v if isinstance(v, str) else ' '.join(v)) for k, v in aria_attrs.items()},
                "text": el.get_text(strip=True)[:60]
            })
    return results


def run_structure_analysis(html_content: str, url: str = "") -> Dict[str, Any]:
    """Main entry point for HTML structure analysis."""
    soup = BeautifulSoup(html_content, 'html.parser')

    # Build DOM tree
    body = soup.find('body') or soup
    tree = _build_element_node(body, 0, max_depth=6)

    # Collect all issues from the tree
    all_issues = []
    def _collect_issues(node: Optional[Dict], path: str = ""):
        if not node:
            return
        current_path = f"{path}/{node['tag']}"
        if node.get('attrs', {}).get('id'):
            current_path += f"#{node['attrs']['id']}"
        for issue in node.get('issues', []):
            all_issues.append({**issue, "path": current_path})
        for child in node.get('children', []):
            _collect_issues(child, current_path)

    _collect_issues(tree)

    # Extract clean text
    clean_text = extract_clean_text(html_content)

    # Extract all IDs and ARIA elements
    all_ids = extract_all_ids(soup)
    all_aria = extract_all_aria(soup)

    # Semantic analysis
    has_main = bool(soup.find('main'))
    has_nav = bool(soup.find('nav'))
    has_header = bool(soup.find('header'))
    has_footer = bool(soup.find('footer'))
    total_divs = len(soup.find_all('div'))
    total_semantic = len(soup.find_all(LANDMARK_TAGS))

    return {
        "url": url,
        "tree": tree,
        "clean_text": clean_text,
        "ids": all_ids,
        "aria_elements": all_aria,
        "issues": all_issues,
        "semantic": {
            "has_main": has_main,
            "has_nav": has_nav,
            "has_header": has_header,
            "has_footer": has_footer,
            "total_divs": total_divs,
            "total_semantic_tags": total_semantic,
            "semantic_ratio": round(total_semantic / max(total_divs, 1) * 100, 1)
        },
        "summary": {
            "total_issues": len(all_issues),
            "total_ids": len(all_ids),
            "total_aria": len(all_aria)
        }
    }
