import re
import math
import logging
from collections import Counter
from typing import Optional

log = logging.getLogger("extractor.nlp")

try:
    import spacy
    _nlp = spacy.load("en_core_web_sm", disable=["parser", "lemmatizer"])
    log.info("Spacy loaded: en_core_web_sm")
except Exception:
    _nlp = None
    log.info("Spacy not available, using rule-based NLP")

from bs4 import BeautifulSoup


STOPWORDS = {
    "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
    "of", "by", "with", "from", "as", "is", "was", "are", "were", "been",
    "be", "has", "have", "had", "do", "does", "did", "will", "would",
    "could", "should", "may", "might", "shall", "can", "need", "dare",
    "this", "that", "these", "those", "it", "its", "they", "them", "their",
    "he", "she", "him", "her", "his", "we", "us", "our", "you", "your",
    "my", "mine", "your", "yours", "not", "no", "nor", "none", "nothing",
    "nobody", "neither", "nor", "all", "each", "every", "both", "few",
    "many", "much", "some", "any", "several", "most", "enough", "such",
    "only", "own", "same", "very", "so", "than", "too", "very", "just",
    "about", "above", "across", "after", "again", "against", "along",
    "among", "around", "before", "behind", "below", "beneath", "beside",
    "between", "beyond", "but", "by", "down", "during", "except", "into",
    "like", "near", "off", "onto", "out", "over", "past", "through",
    "throughout", "to", "toward", "under", "underneath", "until", "upon",
    "within", "without", "also", "else", "further", "moreover", "then",
    "once", "here", "there", "when", "where", "why", "how", "what",
    "which", "who", "whom", "whose", "if", "whether", "because", "since",
    "while", "although", "though", "even", "until", "unless", "except",
}


def _extract_text(html: str) -> str:
    try:
        soup = BeautifulSoup(html, "lxml")
        for tag in soup.find_all(["script", "style", "nav", "footer", "header"]):
            tag.decompose()
        text = soup.get_text(separator=" ", strip=True)
    except Exception:
        text = re.sub(r"<[^>]+>", " ", html)
    return re.sub(r"\s+", " ", text).strip()


def _get_sentences(text: str) -> list[str]:
    raw = re.split(r"(?<=[.!?])\s+", text)
    sentences = []
    for s in raw:
        s = s.strip()
        if len(s) > 15:
            sentences.append(s)
    return sentences


def summarize(text: str, max_sentences: int = 5) -> list[str]:
    sentences = _get_sentences(text)
    if not sentences:
        return []

    if len(sentences) <= max_sentences:
        return sentences[:max_sentences]

    words = re.findall(r"\b[a-zA-Z]{3,}\b", text.lower())
    word_freq = Counter(w for w in words if w not in STOPWORDS and len(w) > 2)
    max_freq = max(word_freq.values()) if word_freq else 1

    scored = []
    for s in sentences:
        score = 0
        s_lower = s.lower()
        for w in re.findall(r"\b[a-zA-Z]{3,}\b", s_lower):
            if w in word_freq:
                score += word_freq[w] / max_freq
        # prefer sentences near the start
        pos_score = 1.0 - (sentences.index(s) / len(sentences))
        scored.append((score + pos_score, s))

    scored.sort(key=lambda x: -x[0])
    top = [s for _, s in scored[:max_sentences]]
    top.sort(key=lambda s: sentences.index(s))
    return top


def extract_entities(text: str) -> dict:
    entities = {"people": [], "organizations": [], "locations": [], "other": []}

    # Named-like entities: capitalized multi-word
    for match in re.finditer(r"\b[A-Z][a-z]+(?:\s+[A-Z][a-z]+)*\b", text):
        word = match.group()
        if len(word) > 4:
            common = {"The", "This", "That", "These", "Those", "What", "When",
                      "Where", "Which", "How", "Why", "Who", "Whom", "Whose"}
            if word not in common:
                entities["other"].append(word)

    # Email-like patterns for people
    for match in re.finditer(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}", text):
        name_part = match.group().split("@")[0].replace(".", " ").replace("_", " ").title()
        if name_part not in entities["people"]:
            entities["people"].append(name_part)

    entities["other"] = list(dict.fromkeys(entities["other"]))[:30]
    entities["people"] = list(dict.fromkeys(entities["people"]))[:20]
    return entities


def extract_keywords(text: str, top_n: int = 30) -> list[dict]:
    words = re.findall(r"\b[a-zA-Z]{3,}\b", text.lower())
    filtered = [w for w in words if w not in STOPWORDS and len(w) > 2]
    freqs = Counter(filtered)
    total = len(filtered) or 1

    # bigrams
    bigrams = Counter()
    for i in range(len(words) - 1):
        bg = f"{words[i]} {words[i+1]}"
        if all(w not in STOPWORDS for w in (words[i], words[i+1])):
            bigrams[bg] += 1

    result = []
    for word, count in freqs.most_common(top_n):
        result.append({
            "word": word,
            "count": count,
            "density": round(count / total * 100, 3),
        })

    top_bigrams = [{"phrase": bg, "count": c}
                   for bg, c in bigrams.most_common(10) if c > 1]

    return {
        "total_words": total,
        "unique_words": len(freqs),
        "keywords": result,
        "bigrams": top_bigrams,
    }


def sentiment_analysis(text: str) -> dict:
    positive_words = {
        "good", "great", "excellent", "amazing", "wonderful", "fantastic",
        "beautiful", "outstanding", "superb", "brilliant", "awesome",
        "impressive", "perfect", "love", "happy", "best", "remarkable",
        "exceptional", "positive", "success", "successful", "beneficial",
        "helpful", "innovative", "powerful", "efficient", "effective",
        "reliable", "secure", "fast", "easy", "intuitive", "smooth",
    }
    negative_words = {
        "bad", "terrible", "awful", "horrible", "worst", "poor",
        "mediocre", "disappointing", "failure", "failed", "fail",
        "broken", "error", "bug", "crash", "slow", "ugly", "difficult",
        "complicated", "confusing", "frustrating", "annoying", "problem",
        "issue", "wrong", "hate", "ugly", "dangerous", "unsafe",
        "expensive", "useless", "inferior", "negative", "damage",
        "vulnerable", "malicious", "attack", "breach", "exploit",
    }

    words = set(re.findall(r"\b[a-zA-Z]{3,}\b", text.lower()))
    positive = words & positive_words
    negative = words & negative_words
    score = (len(positive) - len(negative)) / (len(positive) + len(negative) + 1)

    label = "neutral"
    if score > 0.15:
        label = "positive"
    elif score < -0.15:
        label = "negative"

    return {
        "score": round(score, 3),
        "label": label,
        "positive_words": sorted(positive)[:15],
        "negative_words": sorted(negative)[:15],
    }


def readability(text: str) -> dict:
    sentences = _get_sentences(text)
    words = re.findall(r"\b[a-zA-Z]+\b", text)
    syllables = sum(_count_syllables(w) for w in words)

    num_sentences = len(sentences) or 1
    num_words = len(words) or 1
    num_syllables = syllables or num_words

    # Flesch Reading Ease
    flesch = 206.835 - 1.015 * (num_words / num_sentences) - 84.6 * (num_syllables / num_words)
    flesch = max(0, min(100, flesch))

    level = "very easy"
    if flesch < 30:
        level = "very difficult"
    elif flesch < 50:
        level = "difficult"
    elif flesch < 60:
        level = "fairly difficult"
    elif flesch < 70:
        level = "standard"
    elif flesch < 80:
        level = "fairly easy"
    elif flesch < 90:
        level = "easy"

    return {
        "flesch_score": round(flesch, 1),
        "level": level,
        "sentences": num_sentences,
        "words": num_words,
        "syllables": num_syllables,
        "avg_words_per_sentence": round(num_words / num_sentences, 1),
    }


def _count_syllables(word: str) -> int:
    word = word.lower().strip(".,!?;:")
    if not word:
        return 1
    count = 0
    vowels = "aeiouy"
    prev_is_vowel = False
    for char in word:
        is_vowel = char in vowels
        if is_vowel and not prev_is_vowel:
            count += 1
        prev_is_vowel = is_vowel
    if word.endswith("e"):
        count -= 1
    if word.endswith("le") and len(word) > 2:
        count += 1
    return max(1, count)


def analyze(html: str, url: str) -> dict:
    text = _extract_text(html)
    result = {
        "text_stats": {
            "length": len(text),
            "word_count": len(re.findall(r"\b[a-zA-Z]+\b", text)),
            "sentence_count": len(_get_sentences(text)),
        },
        "summary": summarize(text, 5),
        "entities": extract_entities(text),
        "keywords": extract_keywords(text, 30),
        "sentiment": sentiment_analysis(text),
        "readability": readability(text),
        "method": "rule-based",
    }

    if _nlp:
        try:
            doc = _nlp(text[:50000])
            spacy_entities = {"PERSON": [], "ORG": [], "GPE": [], "DATE": [], "MONEY": [], "OTHER": []}
            for ent in doc.ents:
                label = ent.label_
                group = "OTHER"
                if label in ("PERSON", "PER"):
                    group = "PERSON"
                elif label in ("ORG", "PRODUCT", "WORK_OF_ART"):
                    group = "ORG"
                elif label in ("GPE", "LOC"):
                    group = "GPE"
                elif label == "DATE":
                    group = "DATE"
                elif label == "MONEY":
                    group = "MONEY"
                text_clean = ent.text.strip()
                if len(text_clean) > 1 and text_clean not in spacy_entities[group]:
                    spacy_entities[group].append(text_clean)
            for k in spacy_entities:
                spacy_entities[k] = spacy_entities[k][:20]
            result["entities_spacy"] = spacy_entities
            result["method"] = "spacy"
        except Exception as e:
            log.warning("Spacy analysis failed: %s", e)

    return result
