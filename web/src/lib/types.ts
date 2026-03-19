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

export interface SecurityPerUrl {
  url: string;
  headers?: SecurityHeaders;
  cors?: Record<string, unknown>;
  csp?: { present?: boolean; directives?: Record<string, string[]>; issues?: Array<Record<string, unknown>> };
  sri?: { total_external?: number; missing_integrity?: number };
  mixed_content?: { applicable?: boolean; total_mixed?: number };
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
  mixed_content?: { applicable?: boolean; total_mixed?: number };
  exposure?: { email_count?: number; ip_count?: number; emails?: string[]; internal_ips?: string[] };
  technology?: { technologies?: Array<{ name?: string; source?: string; detail?: string }>; count?: number };
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

export interface ScanReport {
  target_url: string;
  scan_timestamp: string;
  seo: SeoData;
  sitemap: SitemapData;
  security: SecurityData;
  accessibility: AccessibilityData;
  structure: StructureData;
}
