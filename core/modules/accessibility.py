"""
Deep Accessibility Analysis Module.

Checks heading structure, ARIA labels, form validation,
image alt attributes, and language/charset per page.
"""

from bs4 import BeautifulSoup
from typing import Dict, List, Any, Optional


def analyze_heading_structure(soup: BeautifulSoup) -> Dict[str, Any]:
    """Checks H1 presence, <p> following headings, and hierarchy."""
    issues = []
    headings_detail = []

    all_headings = soup.find_all(['h1', 'h2', 'h3', 'h4', 'h5', 'h6'])
    h1_tags = soup.find_all('h1')

    if not h1_tags:
        issues.append({"severity": "error", "message": "No H1 tag found on the page"})
    elif len(h1_tags) > 1:
        issues.append({"severity": "warning", "message": f"Multiple H1 tags found ({len(h1_tags)})"})

    prev_level = 0
    for heading in all_headings:
        tag_name = heading.name
        level = int(tag_name[1])
        text = heading.get_text(strip=True)[:120]

        # Check hierarchy skip (e.g. H1 -> H3 without H2)
        if prev_level > 0 and level > prev_level + 1:
            issues.append({
                "severity": "warning",
                "message": f"Heading hierarchy skip: {f'H{prev_level}'} -> {tag_name.upper()} (missing {f'H{prev_level + 1}'})"
            })

        # Check if heading has <p> content below it
        next_sib = heading.find_next_sibling()
        has_content_below = False
        if next_sib:
            # Walk siblings until next heading or end
            sib = next_sib
            while sib and sib.name not in ['h1', 'h2', 'h3', 'h4', 'h5', 'h6']:
                if sib.name in ['p', 'div', 'ul', 'ol', 'section', 'article', 'table']:
                    sib_text = sib.get_text(strip=True)
                    if sib_text:
                        has_content_below = True
                        break
                sib = sib.find_next_sibling()

        if not has_content_below:
            issues.append({
                "severity": "info",
                "message": f"{tag_name.upper()} \"{text[:60]}\" has no paragraph or content block below it"
            })

        headings_detail.append({
            "tag": tag_name.upper(),
            "level": level,
            "text": text,
            "has_content_below": has_content_below
        })

        prev_level = level

    return {
        "total_headings": len(all_headings),
        "h1_count": len(h1_tags),
        "headings": headings_detail,
        "issues": issues
    }


def analyze_image_accessibility(soup: BeautifulSoup) -> Dict[str, Any]:
    """Checks every <img> for alt attribute presence and quality."""
    images = soup.find_all('img')
    results = []
    missing_count = 0
    empty_count = 0

    for img in images:
        src = img.get('src', 'unknown')
        alt = img.get('alt')
        has_alt = alt is not None
        is_empty = has_alt and alt.strip() == ''

        if not has_alt:
            missing_count += 1
        elif is_empty:
            empty_count += 1

        results.append({
            "src": src[:200],
            "has_alt": has_alt,
            "alt_empty": is_empty,
            "alt_text": (alt[:100] if alt else None)
        })

    return {
        "total_images": len(images),
        "missing_alt": missing_count,
        "empty_alt": empty_count,
        "images": results
    }


def analyze_aria_labels(soup: BeautifulSoup) -> Dict[str, Any]:
    """Checks interactive elements for proper ARIA labeling."""
    interactive_tags = ['button', 'a', 'input', 'select', 'textarea']
    issues = []
    total_checked = 0
    missing_label = 0

    for tag_name in interactive_tags:
        elements = soup.find_all(tag_name)
        for el in elements:
            total_checked += 1

            has_aria_label = bool(el.get('aria-label'))
            has_aria_labelledby = bool(el.get('aria-labelledby'))
            has_title = bool(el.get('title'))

            # For inputs, check associated <label>
            has_associated_label = False
            if tag_name in ['input', 'select', 'textarea']:
                el_id = el.get('id')
                if el_id:
                    has_associated_label = bool(soup.find('label', attrs={'for': el_id}))
                # Also check if wrapped in <label>
                if not has_associated_label:
                    parent = el.find_parent('label')
                    if parent:
                        has_associated_label = True

            # For buttons/links, check visible text content
            visible_text = el.get_text(strip=True) if tag_name in ['button', 'a'] else ''

            is_labeled = (
                has_aria_label or
                has_aria_labelledby or
                has_title or
                has_associated_label or
                bool(visible_text)
            )

            # Skip hidden inputs
            if tag_name == 'input' and el.get('type') == 'hidden':
                total_checked -= 1
                continue

            if not is_labeled:
                missing_label += 1
                el_str = str(el)[:150]
                issues.append({
                    "severity": "warning",
                    "element": tag_name,
                    "html_snippet": el_str,
                    "message": f"<{tag_name}> element missing accessible label (no aria-label, aria-labelledby, title, or visible text)"
                })

    return {
        "total_interactive": total_checked,
        "missing_labels": missing_label,
        "issues": issues
    }


def analyze_forms(soup: BeautifulSoup) -> Dict[str, Any]:
    """Validates form structure: labels, inputs, submit buttons."""
    forms = soup.find_all('form')
    results = []

    for idx, form in enumerate(forms):
        form_issues = []
        action = form.get('action', '')
        method = form.get('method', 'GET').upper()

        inputs = form.find_all(['input', 'select', 'textarea'])
        labeled_inputs = 0
        unlabeled_inputs = []

        for inp in inputs:
            if inp.get('type') == 'hidden':
                continue

            inp_id = inp.get('id')
            inp_name = inp.get('name', '')
            has_label = False

            # Check for <label for="id">
            if inp_id and form.find('label', attrs={'for': inp_id}):
                has_label = True
            # Check if wrapped in <label>
            if not has_label and inp.find_parent('label'):
                has_label = True
            # Check aria-label
            if not has_label and (inp.get('aria-label') or inp.get('placeholder')):
                has_label = True

            if has_label:
                labeled_inputs += 1
            else:
                unlabeled_inputs.append({
                    "tag": inp.name,
                    "name": inp_name,
                    "type": inp.get('type', 'text'),
                    "html": str(inp)[:120]
                })
                form_issues.append({
                    "severity": "warning",
                    "message": f"Input '{inp_name or inp.get('type', 'text')}' has no associated label"
                })

            if not inp_name and not inp_id:
                form_issues.append({
                    "severity": "info",
                    "message": f"Input element has no 'name' or 'id' attribute"
                })

        # Check for submit button
        has_submit = bool(
            form.find('button', attrs={'type': 'submit'}) or
            form.find('input', attrs={'type': 'submit'}) or
            form.find('button', attrs={'type': None})  # default type=submit
        )
        if not has_submit:
            form_issues.append({
                "severity": "warning",
                "message": "Form has no submit button"
            })

        total_visible_inputs = len([i for i in inputs if i.get('type') != 'hidden'])

        results.append({
            "index": idx + 1,
            "action": action,
            "method": method,
            "total_inputs": total_visible_inputs,
            "labeled_inputs": labeled_inputs,
            "unlabeled_inputs": unlabeled_inputs,
            "has_submit": has_submit,
            "issues": form_issues
        })

    return {
        "total_forms": len(forms),
        "forms": results
    }


def analyze_language_and_charset(soup: BeautifulSoup, html_content: str) -> Dict[str, Any]:
    """Validates <html lang> and <meta charset> presence and consistency."""
    issues = []

    # Check <html lang>
    html_tag = soup.find('html')
    lang = html_tag.get('lang', '') if html_tag else ''
    has_lang = bool(lang)

    if not has_lang:
        issues.append({
            "severity": "error",
            "message": "Missing 'lang' attribute on <html> tag"
        })

    # Check <meta charset>
    charset_meta = soup.find('meta', attrs={'charset': True})
    charset_http = None
    charset_tag = None

    if charset_meta:
        charset_tag = charset_meta.get('charset', '').upper()
    else:
        # Check http-equiv content-type
        ct_meta = soup.find('meta', attrs={'http-equiv': lambda x: x and x.lower() == 'content-type'})
        if ct_meta:
            content = ct_meta.get('content', '')
            if 'charset=' in content.lower():
                charset_tag = content.lower().split('charset=')[1].strip().upper()

    has_charset = charset_tag is not None

    if not has_charset:
        issues.append({
            "severity": "error",
            "message": "Missing <meta charset> declaration"
        })
    elif charset_tag not in ['UTF-8', 'UTF8']:
        issues.append({
            "severity": "warning",
            "message": f"Charset is '{charset_tag}' -- UTF-8 is recommended for maximum compatibility"
        })

    # Check if content has encoding issues (basic heuristic)
    encoding_ok = True
    try:
        html_content.encode('utf-8').decode('utf-8')
    except (UnicodeDecodeError, UnicodeEncodeError):
        encoding_ok = False
        issues.append({
            "severity": "error",
            "message": "Content has encoding issues -- characters may not display correctly"
        })

    return {
        "lang": lang if has_lang else None,
        "has_lang": has_lang,
        "charset": charset_tag,
        "has_charset": has_charset,
        "encoding_valid": encoding_ok,
        "issues": issues
    }


def run_accessibility_analysis(html_content: str, url: str = "") -> Dict[str, Any]:
    """Main entry point for accessibility analysis on a single page."""
    soup = BeautifulSoup(html_content, 'html.parser')

    heading_data = analyze_heading_structure(soup)
    image_data = analyze_image_accessibility(soup)
    aria_data = analyze_aria_labels(soup)
    form_data = analyze_forms(soup)
    lang_data = analyze_language_and_charset(soup, html_content)

    # Aggregate all issues
    all_issues = (
        heading_data["issues"] +
        image_data.get("issues", []) +
        aria_data["issues"] +
        lang_data["issues"]
    )
    for f in form_data["forms"]:
        all_issues.extend(f["issues"])

    error_count = len([i for i in all_issues if i.get("severity") == "error"])
    warning_count = len([i for i in all_issues if i.get("severity") == "warning"])
    info_count = len([i for i in all_issues if i.get("severity") == "info"])

    return {
        "url": url,
        "headings": heading_data,
        "images": image_data,
        "aria": aria_data,
        "forms": form_data,
        "language": lang_data,
        "summary": {
            "total_issues": len(all_issues),
            "errors": error_count,
            "warnings": warning_count,
            "info": info_count
        }
    }
