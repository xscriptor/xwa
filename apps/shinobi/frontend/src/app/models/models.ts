export interface ScrapeConfig {
    url: string;
    depth?: number;
    concurrency?: number;
    delay_ms?: number;
    max_pages?: number;
    same_domain_only?: boolean;
    download_assets?: boolean;
    user_agent_rotation?: boolean;
    javascript_rendering?: boolean;
    file_types?: string;
    respect_robots_txt?: boolean;
    deduplicate?: boolean;
    take_screenshots?: boolean;
    extract_emails?: boolean;
    webhook_url?: string;
    rewrite_urls?: boolean;
    generate_index?: boolean;
    export_warc?: boolean;
    auth_username?: string;
    auth_password?: string;
    auth_mode?: string;
    rate_limit?: number;
    deep_mode?: boolean;
    extract_structured?: boolean;
    nlp_enabled?: boolean;
    custom_selectors?: string[];
}

export interface JobInfo {
    id: string;
    url: string;
    status: string;
    created_at: string;
    pages_scraped: number;
    files_downloaded: number;
    total_pages: number;
    current_url: string | null;
    errors: string[];
    emails: string[];
    phones: string[];
}

export interface FileInfo {
    name: string;
    path: string;
    is_dir: boolean;
    size: number;
    modified: number;
}

export interface DeepConfig {
    url: string;
    extract_structured?: boolean;
    nlp_enabled?: boolean;
    custom_selectors?: string[];
}

export interface DeepResult {
    id: string;
    job_id: string;
    url: string;
    structured_data: any;
    nlp_data: any;
    extracted: {
        emails: string[];
        phones: string[];
    };
    created_at: string;
}

export interface DeepBatchQuery {
    urls: string[];
    extract_structured?: boolean;
    nlp_enabled?: boolean;
    custom_selectors?: string[];
}

export interface PaginatedResponse<T> {
    items: T[];
    total: number;
    offset: number;
    limit: number;
}
