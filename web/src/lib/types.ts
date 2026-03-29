export interface SeoStandardMeta {
  title?: string | null;
  description?: string | null;
  keywords?: string | null;
  author?: string | null;
  robots?: string | null;
  viewport?: string | null;
  charset?: string | null;
  generator?: string | null;
  theme_color?: string | null;
}

export interface SeoHeadings {
  counts?: Record<string, number>;
  details?: Array<{ tag?: string; text?: string; id?: string }>;
  missing_h1?: boolean;
  multiple_h1?: boolean;
}

export interface SeoImageAlts {
  total_images?: number;
  missing_alt?: number;
  missing_alt_urls?: string[];
}

export interface SeoTextRatio {
  html_size_bytes?: number;
  text_size_bytes?: number;
  text_to_html_ratio?: number;
  word_count?: number;
}

export interface SeoPerUrl {
  url: string;
  standard_meta?: SeoStandardMeta;
  social_meta?: { og?: Record<string, string>; twitter?: Record<string, string> };
  structured_data?: { json_ld?: Array<Record<string, unknown>>; microdata_types?: string[] };
  link_tags?: Array<Record<string, string>>;
  headings?: SeoHeadings;
  image_alts?: SeoImageAlts;
  text_ratio?: SeoTextRatio;
  canonical?: string | null;
}

export interface SeoData {
  standard_meta?: SeoStandardMeta;
  social_meta?: { og?: Record<string, string>; twitter?: Record<string, string> };
  headings?: SeoHeadings;
  image_alts?: SeoImageAlts;
  text_ratio?: SeoTextRatio;
  canonical?: string | null;
  robots_txt?: {
    presence?: boolean;
    url?: string;
    status_code?: number;
    directives?: string[];
    sitemaps?: string[];
  };
  structured_data?: { json_ld?: Array<Record<string, unknown>>; microdata_types?: string[] };
  link_tags?: Array<Record<string, string>>;
  per_url?: SeoPerUrl[];
}

export interface SecurityHeaders {
  headers_present?: Record<string, string>;
  missing_headers?: string[];
  leaked_server_info?: Record<string, string>;
}

export interface SecuritySsl {
  valid?: boolean;
  issuer?: string;
  subject?: string;
  expires_on?: string;
  days_remaining?: number;
  is_expired?: boolean;
  error?: string;
}

export interface SecurityCookies {
  total?: number;
  cookies?: Array<{ name?: string; secure?: boolean; httponly?: boolean; samesite?: string }>;
  issues?: string[];
}

export interface SecurityMixedContentResource {
  tag?: string;
  attribute?: string;
  url?: string;
}

export interface SecurityVulnerabilityCve {
  id?: string;
  published?: string;
  last_modified?: string;
  severity?: string;
  cvss_score?: number;
  description?: string;
}

export interface SecurityVulnerabilityFinding {
  technology?: string;
  keyword?: string;
  version?: string;
  count?: number;
  cves?: SecurityVulnerabilityCve[];
}

export interface SecurityVulnerabilities {
  available?: boolean;
  provider?: string;
  status?: string;
  total_cves?: number;
  findings?: SecurityVulnerabilityFinding[];
  errors?: string[];
}

export interface SecurityBlacklistProvider {
  provider?: string;
  available?: boolean;
  listed?: boolean | null;
  status?: string;
  reason?: string;
  error?: string;
  threat?: string | null;
  url_status?: string | null;
  tags?: string[];
}

export interface SecurityBlacklist {
  status?: string;
  is_listed?: boolean;
  providers?: SecurityBlacklistProvider[];
}

export interface SecurityDnsSecurity {
  domain?: string | null;
  status?: string;
  error?: string;
  dnssec?: { present?: boolean; dnskey_records?: string[]; rrsig_records?: string[]; error?: string | null };
  spf?: { present?: boolean; records?: string[] };
  dkim?: { present?: boolean; selectors_checked?: string[]; records?: Array<{ selector?: string; record?: string }> };
  dmarc?: { present?: boolean; records?: string[] };
  summary?: { configured?: string[]; missing?: string[] };
}

export interface SecurityPerUrl {
  url: string;
  headers?: SecurityHeaders;
  cors?: Record<string, unknown>;
  csp?: { present?: boolean; directives?: Record<string, string[]>; issues?: Array<Record<string, unknown>> };
  sri?: { total_external?: number; missing_integrity?: number };
  mixed_content?: { applicable?: boolean; total_mixed?: number; resources?: SecurityMixedContentResource[] };
  exposure?: { email_count?: number; ip_count?: number; emails?: string[]; internal_ips?: string[] };
  technology?: { technologies?: Array<{ name?: string; source?: string; detail?: string }>; count?: number };
}

export interface SecurityData {
  headers?: SecurityHeaders;
  ssl?: SecuritySsl;
  cookies?: SecurityCookies;
  sensitive_paths_found?: string[];
  cors?: Record<string, unknown>;
  csp?: { present?: boolean; directives?: Record<string, string[]>; issues?: Array<Record<string, unknown>> };
  sri?: { total_external?: number; missing_integrity?: number };
  mixed_content?: { applicable?: boolean; total_mixed?: number; resources?: SecurityMixedContentResource[] };
  exposure?: { email_count?: number; ip_count?: number; emails?: string[]; internal_ips?: string[] };
  technology?: { technologies?: Array<{ name?: string; source?: string; detail?: string }>; count?: number };
  vulnerabilities?: SecurityVulnerabilities;
  blacklist?: SecurityBlacklist;
  dns_security?: SecurityDnsSecurity;
  http_methods?: { allowed?: string[]; dangerous?: string[]; has_dangerous?: boolean };
  per_url?: SecurityPerUrl[];
}

export interface SitemapCrawlResult {
  url: string;
  status?: number;
  ok?: boolean;
  response_time_ms?: number;
  title?: string;
  description?: string;
  h1?: string;
  word_count?: number;
  canonical?: string;
  error?: string;
}

export interface SitemapData {
  urls_found?: number;
  scanned_count?: number;
  broken_links?: SitemapCrawlResult[];
  all_urls?: string[];
  tree_root_children?: string[];
  crawl_results?: SitemapCrawlResult[];
  avg_response_time_ms?: number;
}

export interface AccessibilityPage {
  url?: string;
  headings?: {
    total_headings?: number;
    headings?: Array<{ tag?: string; text?: string; has_content_below?: boolean }>;
    issues?: Array<{ severity: string; message: string }>;
  };
  images?: {
    total_images?: number;
    missing_alt?: number;
    empty_alt?: number;
    images?: Array<{ src?: string; has_alt?: boolean; alt_empty?: boolean; alt_text?: string | null }>;
  };
  aria?: {
    total_interactive?: number;
    missing_labels?: number;
    issues?: Array<{ severity: string; message: string; html_snippet?: string }>;
  };
  forms?: {
    total_forms?: number;
    forms?: Array<{
      index?: number;
      action?: string;
      method?: string;
      total_inputs?: number;
      labeled_inputs?: number;
      has_submit?: boolean;
      issues?: Array<{ severity: string; message: string }>;
      unlabeled_inputs?: Array<{ tag?: string; type?: string; name?: string; html?: string }>;
    }>;
  };
  language?: {
    lang?: string;
    has_lang?: boolean;
    charset?: string;
    has_charset?: boolean;
    encoding_valid?: boolean;
    issues?: Array<{ severity: string; message: string }>;
  };
  summary?: {
    total_issues?: number;
    errors?: number;
    warnings?: number;
    info?: number;
  };
}

export interface AccessibilityData {
  main_page?: AccessibilityPage;
  per_url?: AccessibilityPage[];
}

export interface StructurePage {
  url?: string;
  tree?: Record<string, unknown>;
  clean_text?: {
    total_chars?: number;
    total_words?: number;
    sections?: Array<{ tag?: string; id?: string; text?: string }>;
    preview?: string;
  };
  ids?: Array<{ tag?: string; id?: string; text?: string }>;
  aria_elements?: Array<{ tag?: string; id?: string; aria?: Record<string, string>; text?: string }>;
  issues?: Array<{ type: string; message: string; path?: string }>;
  semantic?: {
    has_main?: boolean;
    has_nav?: boolean;
    has_header?: boolean;
    has_footer?: boolean;
    total_divs?: number;
    total_semantic_tags?: number;
    semantic_ratio?: number;
  };
  summary?: {
    total_issues?: number;
    total_ids?: number;
    total_aria?: number;
  };
}

export interface StructureData {
  main_page?: StructurePage;
  per_url?: StructurePage[];
}

export interface PerformanceTTFB {
  ttfb_estimate_ms?: number;
  transfer_estimate_ms?: number;
  total_response_ms?: number;
  html_size_bytes?: number;
  rating?: string;
}

export interface PerformanceResources {
  total_external_requests?: number;
  html_size_bytes?: number;
  js?: { external_count?: number; inline_count?: number; sources?: string[] };
  css?: { external_count?: number; inline_count?: number; sources?: string[] };
  images?: { count?: number; sources?: string[] };
  fonts?: { count?: number; sources?: string[] };
  iframes?: { count?: number; sources?: string[] };
}

export interface PerformanceCWV {
  lcp?: { estimate_ms?: number; rating?: string; blocking_js?: number; blocking_css?: number; hint?: string };
  cls?: { risk?: string; unstable_elements?: number; hint?: string };
  inp?: { risk?: string; sync_scripts?: number; hint?: string };
}

export interface PerformanceUnoptimizedImages {
  total_images?: number;
  unoptimized_count?: number;
  unoptimized?: Array<{ src?: string; format?: string }>;
  has_picture_element_modern?: boolean;
  recommendation?: string;
}

export interface PerformancePage {
  url?: string;
  ttfb?: PerformanceTTFB;
  resources?: PerformanceResources;
  unoptimized_images?: PerformanceUnoptimizedImages;
  cwv_estimates?: PerformanceCWV;
}

export interface PerformanceData {
  main_page?: PerformancePage;
  per_url?: PerformancePage[];
}

export interface ScanReport {
  target_url: string;
  scan_timestamp: string;
  seo: SeoData;
  sitemap: SitemapData;
  security: SecurityData;
  accessibility: AccessibilityData;
  structure: StructureData;
  performance: PerformanceData;
}
