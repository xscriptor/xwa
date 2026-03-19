#!/usr/bin/env node

const cheerio = require("cheerio");
const fetch = require("node-fetch");
const { URL } = require("url");
const fs = require("fs");
const path = require("path");

// --- Configuracion ---
const MAX_CONCURRENCY = 5;
const REQUEST_DELAY_MS = 200;
const REQUEST_TIMEOUT_MS = 15000;
const MAX_PAGES = 50000;
const USER_AGENT =
  "Mozilla/5.0 (compatible; SitemapCrawler/2.0; +https://example.com/bot)";

// Patrones a excluir del sitemap
const EXCLUDE_PATTERNS = [
  /\/author\//i,
  /\/feed\/?$/i,
  /\/feed\/atom\/?$/i,
  /\/feed\/rss\/?$/i,
  /\/comment-page-\d+/i,
  /\/attachment\//i,
  /\/(wp-content|wp-admin|wp-includes|wp-json)\//i,
  /[?&](upage|replytocom|preview|share|doing_wp_cron)=/i,
  /\/(login|register|cart|checkout|my-account)\/?$/i,
  /\/xmlrpc\.php/i,
  /\/trackback\/?$/i,
  /\/embed\/?$/i,
  /\?p=\d+$/i,
  /\/page\/\d+\/?$/i,
  /\.(jpg|jpeg|png|gif|svg|webp|ico|pdf|doc|docx|xls|xlsx|ppt|zip|rar|css|js|json|xml|rss|woff|woff2|ttf|mp3|mp4|avi|mov)(\?|$)/i,
  /^(mailto:|tel:|javascript:|data:|ftp:)/i,
];

// --- Estado del crawler ---
const visited = new Set();
const failed = new Set();
const queue = [];
let activeRequests = 0;

// Datos enriquecidos por URL
const urlData = new Map();

function normalizeUrl(rawUrl, base) {
  try {
    const parsed = new URL(rawUrl, base);
    parsed.hash = "";
    // Eliminar parametros de tracking comunes
    ["utm_source", "utm_medium", "utm_campaign", "utm_term", "utm_content",
     "fbclid", "gclid", "mc_cid", "mc_eid", "ref", "source"].forEach(p => parsed.searchParams.delete(p));
    let href = parsed.href;
    // Normalizar trailing slash: siempre CON slash (excepto si tiene extension)
    if (parsed.pathname !== "/" && !parsed.pathname.match(/\.\w{2,5}$/) && !href.endsWith("/")) {
      href = href + "/";
    }
    return href;
  } catch {
    return null;
  }
}

function isSameDomain(url, origin) {
  try {
    return new URL(url).hostname === new URL(origin).hostname;
  } catch {
    return false;
  }
}

function shouldSkip(url) {
  return EXCLUDE_PATTERNS.some((re) => re.test(url));
}

function getDepth(url) {
  try {
    const pathname = new URL(url).pathname.replace(/\/$/, "");
    if (pathname === "" || pathname === "/") return 0;
    return pathname.split("/").filter(Boolean).length;
  } catch {
    return 99;
  }
}

function calculatePriority(url, origin, data) {
  const depth = getDepth(url);
  // Home = 1.0
  if (url === origin || url === origin + "/") return 1.0;
  // Depth-based with bonus for pages with many inlinks
  let priority = Math.max(0.1, 1.0 - depth * 0.2);
  // Bonus if many other pages link to this
  if (data.inlinks && data.inlinks > 10) priority = Math.min(1.0, priority + 0.1);
  if (data.inlinks && data.inlinks > 50) priority = Math.min(1.0, priority + 0.1);
  return Math.round(priority * 10) / 10;
}

function guessChangefreq(url, data) {
  const depth = getDepth(url);
  if (depth === 0) return "daily";
  if (depth === 1) return "weekly";
  // Blog posts with dates in URL are less likely to change
  if (/\/\d{4}\/\d{2}\//.test(url)) return "monthly";
  // Event pages
  if (/\/evento\//i.test(url)) return "weekly";
  return "weekly";
}

function escapeXml(str) {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&apos;");
}

function delay(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

async function fetchPage(url) {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS);
  const startTime = Date.now();

  try {
    const res = await fetch(url, {
      headers: {
        "User-Agent": USER_AGENT,
        Accept: "text/html,application/xhtml+xml",
        "Accept-Language": "es,en;q=0.9",
      },
      signal: controller.signal,
      redirect: "manual", // No seguir redirecciones automaticamente
      follow: 0,
    });
    clearTimeout(timeout);

    const responseTime = Date.now() - startTime;
    const status = res.status;
    const finalUrl = res.headers.get("location");

    // Redireccion
    if (status >= 300 && status < 400 && finalUrl) {
      const resolvedRedirect = normalizeUrl(finalUrl, url);
      return {
        html: null,
        status,
        responseTime,
        redirectTo: resolvedRedirect,
        finalUrl: resolvedRedirect,
      };
    }

    const contentType = res.headers.get("content-type") || "";
    if (!contentType.includes("text/html")) {
      return { html: null, status, responseTime, redirectTo: null, finalUrl: url };
    }

    if (!res.ok) {
      return { html: null, status, responseTime, redirectTo: null, finalUrl: url };
    }

    const html = await res.text();
    return { html, status, responseTime, redirectTo: null, finalUrl: url };
  } catch (err) {
    clearTimeout(timeout);
    return {
      html: null,
      status: err.name === "AbortError" ? 408 : 0,
      responseTime: Date.now() - startTime,
      redirectTo: null,
      finalUrl: url,
    };
  }
}

function extractPageData(html, baseUrl) {
  const $ = cheerio.load(html);
  const links = new Set();
  const data = {};

  // Titulo
  data.title = $("title").first().text().trim().substring(0, 200) || "";

  // Meta description
  data.description =
    $('meta[name="description"]').attr("content")?.trim().substring(0, 300) || "";

  // Canonical
  const canonical = $('link[rel="canonical"]').attr("href");
  if (canonical) {
    data.canonical = normalizeUrl(canonical, baseUrl);
  }

  // Meta robots
  const metaRobots = $('meta[name="robots"]').attr("content") || "";
  data.noindex = /noindex/i.test(metaRobots);
  data.nofollow = /nofollow/i.test(metaRobots);

  // Hreflang
  data.hreflang = {};
  $('link[rel="alternate"][hreflang]').each((_, el) => {
    const lang = $(el).attr("hreflang");
    const href = $(el).attr("href");
    if (lang && href) {
      data.hreflang[lang] = normalizeUrl(href, baseUrl);
    }
  });

  // H1
  data.h1 = $("h1").first().text().trim().substring(0, 200) || "";

  // Word count (aproximado)
  const bodyText = $("body").text().replace(/\s+/g, " ").trim();
  data.wordCount = bodyText.split(" ").filter((w) => w.length > 0).length;

  // Last modified header or meta
  data.lastModified =
    $('meta[property="article:modified_time"]').attr("content") ||
    $('meta[property="og:updated_time"]').attr("content") ||
    "";

  // Open Graph image
  data.ogImage = $('meta[property="og:image"]').attr("content") || "";

  // Extract links
  $("a[href]").each((_, el) => {
    const href = $(el).attr("href");
    if (!href) return;
    const normalized = normalizeUrl(href, baseUrl);
    if (!normalized) return;
    if (!isSameDomain(normalized, baseUrl)) return;
    if (shouldSkip(normalized)) return;
    links.add(normalized);
  });

  return { links, data };
}

// Contar inlinks
const inlinkCount = new Map();

async function processUrl(url, origin) {
  if (visited.has(url) || visited.size >= MAX_PAGES) return;
  visited.add(url);

  const result = await fetchPage(url);

  // Guardar datos basicos
  const entry = {
    status: result.status,
    responseTime: result.responseTime,
    redirectTo: result.redirectTo,
  };

  // Si es redireccion, registrar pero no incluir en sitemap
  if (result.redirectTo) {
    entry.isRedirect = true;
    urlData.set(url, entry);
    // Agregar destino de redireccion al queue
    if (
      result.redirectTo &&
      isSameDomain(result.redirectTo, origin) &&
      !visited.has(result.redirectTo) &&
      !shouldSkip(result.redirectTo)
    ) {
      queue.push(result.redirectTo);
    }
    return;
  }

  if (!result.html) {
    failed.add(url);
    visited.delete(url);
    entry.failed = true;
    urlData.set(url, entry);
    return;
  }

  const { links, data } = extractPageData(result.html, url);
  Object.assign(entry, data);
  urlData.set(url, entry);

  // Si tiene canonical diferente, marcar
  if (data.canonical && data.canonical !== url) {
    entry.hasCanonicalMismatch = true;
    entry.canonicalTarget = data.canonical;
  }

  // Contar inlinks
  for (const link of links) {
    inlinkCount.set(link, (inlinkCount.get(link) || 0) + 1);
    if (
      !visited.has(link) &&
      !queue.includes(link) &&
      isSameDomain(link, origin) &&
      !shouldSkip(link)
    ) {
      queue.push(link);
    }
  }
}

async function worker(origin) {
  while (queue.length > 0 || activeRequests > 0) {
    if (queue.length === 0) {
      await delay(100);
      continue;
    }
    if (visited.size >= MAX_PAGES) break;

    const url = queue.shift();
    if (visited.has(url)) continue;

    activeRequests++;
    process.stdout.write(
      `\r  Rastreadas: ${visited.size} | Cola: ${queue.length} | Fallidas: ${failed.size}  `
    );

    await processUrl(url, origin);
    activeRequests--;
    await delay(REQUEST_DELAY_MS);
  }
}

function generateSitemap(origin) {
  // Filtrar URLs para el sitemap
  const sitemapUrls = [];

  for (const [url, data] of urlData) {
    // Excluir redirecciones
    if (data.isRedirect) continue;
    // Excluir fallidas
    if (data.failed) continue;
    // Excluir noindex
    if (data.noindex) continue;
    // Excluir si el canonical apunta a otra URL
    if (data.hasCanonicalMismatch) continue;
    // Excluir patrones sospechosos
    if (shouldSkip(url)) continue;
    // Excluir status != 200
    if (data.status !== 200) continue;

    // Calcular inlinks
    data.inlinks = inlinkCount.get(url) || 0;

    sitemapUrls.push({ url, data });
  }

  // Ordenar por prioridad descendente, luego alfabeticamente
  sitemapUrls.sort((a, b) => {
    const pa = calculatePriority(a.url, origin, a.data);
    const pb = calculatePriority(b.url, origin, b.data);
    if (pb !== pa) return pb - pa;
    return a.url.localeCompare(b.url);
  });

  const today = new Date().toISOString().split("T")[0];

  let xml = '<?xml version="1.0" encoding="UTF-8"?>\n';
  xml += '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n';

  for (const { url, data } of sitemapUrls) {
    const priority = calculatePriority(url, origin, data);
    const changefreq = guessChangefreq(url, data);
    const lastmod = data.lastModified
      ? data.lastModified.split("T")[0]
      : today;

    xml += "  <url>\n";
    xml += `    <loc>${escapeXml(url)}</loc>\n`;
    xml += `    <lastmod>${lastmod}</lastmod>\n`;
    xml += `    <changefreq>${changefreq}</changefreq>\n`;
    xml += `    <priority>${priority}</priority>\n`;
    xml += "  </url>\n";
  }

  xml += "</urlset>\n";
  return { xml, count: sitemapUrls.length, urls: sitemapUrls };
}

function generateReport(origin, sitemapUrls) {
  const report = {
    meta: {
      url: origin,
      date: new Date().toISOString(),
      totalCrawled: visited.size,
      totalFailed: failed.size,
      totalInSitemap: sitemapUrls.length,
    },
    redirects: [],
    noindex: [],
    canonicalMismatches: [],
    slow: [],
    noTitle: [],
    noDescription: [],
    noH1: [],
    thinContent: [],
    hreflang: {},
    statusCodes: {},
    depthDistribution: {},
    avgResponseTime: 0,
  };

  let totalTime = 0;
  let timeCount = 0;

  for (const [url, data] of urlData) {
    // Status codes
    const status = data.status || 0;
    report.statusCodes[status] = (report.statusCodes[status] || 0) + 1;

    // Response time
    if (data.responseTime) {
      totalTime += data.responseTime;
      timeCount++;
    }

    // Depth
    const depth = getDepth(url);
    report.depthDistribution[depth] = (report.depthDistribution[depth] || 0) + 1;

    // Redirects
    if (data.isRedirect) {
      report.redirects.push({ from: url, to: data.redirectTo, status: data.status });
    }

    // Noindex
    if (data.noindex) {
      report.noindex.push(url);
    }

    // Canonical mismatches
    if (data.hasCanonicalMismatch) {
      report.canonicalMismatches.push({ url, canonical: data.canonicalTarget });
    }

    // Slow pages (> 3s)
    if (data.responseTime > 3000 && data.status === 200) {
      report.slow.push({ url, time: data.responseTime });
    }

    // Missing title
    if (data.status === 200 && !data.isRedirect && !data.title) {
      report.noTitle.push(url);
    }

    // Missing description
    if (data.status === 200 && !data.isRedirect && !data.description) {
      report.noDescription.push(url);
    }

    // Missing H1
    if (data.status === 200 && !data.isRedirect && !data.h1) {
      report.noH1.push(url);
    }

    // Thin content (< 100 words)
    if (data.status === 200 && !data.isRedirect && data.wordCount && data.wordCount < 100) {
      report.thinContent.push({ url, words: data.wordCount });
    }

    // Hreflang
    if (data.hreflang && Object.keys(data.hreflang).length > 0) {
      report.hreflang[url] = data.hreflang;
    }
  }

  report.avgResponseTime = timeCount > 0 ? Math.round(totalTime / timeCount) : 0;
  report.slow.sort((a, b) => b.time - a.time);

  return report;
}

async function tryExistingSitemap(origin) {
  console.log("  Buscando sitemap existente...");
  const urls = new Set();

  const candidates = [
    `${origin}/sitemap.xml`,
    `${origin}/sitemap_index.xml`,
    `${origin}/sitemap/`,
  ];

  for (const url of candidates) {
    try {
      const res = await fetch(url, {
        headers: { "User-Agent": USER_AGENT },
        timeout: 10000,
      });
      if (res.ok) {
        const text = await res.text();
        if (text.includes("<urlset") || text.includes("<sitemapindex")) {
          console.log(`  Encontrado sitemap en: ${url}`);
          const $ = cheerio.load(text, { xmlMode: true });

          const sitemapLocs = [];
          $("sitemap > loc").each((_, el) => sitemapLocs.push($(el).text().trim()));

          if (sitemapLocs.length > 0) {
            console.log(`  Indice con ${sitemapLocs.length} sub-sitemaps`);
            for (const loc of sitemapLocs) {
              try {
                const subRes = await fetch(loc, {
                  headers: { "User-Agent": USER_AGENT },
                  timeout: 10000,
                });
                if (subRes.ok) {
                  const subText = await subRes.text();
                  const $sub = cheerio.load(subText, { xmlMode: true });
                  $sub("url > loc").each((_, el) => urls.add($sub(el).text().trim()));
                }
              } catch {}
            }
          }

          $("url > loc").each((_, el) => urls.add($(el).text().trim()));
        }
      }
    } catch {}
  }

  // Robots.txt
  try {
    const robotsRes = await fetch(`${origin}/robots.txt`, {
      headers: { "User-Agent": USER_AGENT },
    });
    if (robotsRes.ok) {
      const text = await robotsRes.text();
      const sitemapLines = text
        .split("\n")
        .filter((l) => l.toLowerCase().startsWith("sitemap:"));
      for (const line of sitemapLines) {
        const sitemapUrl = line.split(":").slice(1).join(":").trim();
        if (sitemapUrl && !candidates.includes(sitemapUrl)) {
          console.log(`  Sitemap en robots.txt: ${sitemapUrl}`);
          try {
            const res = await fetch(sitemapUrl, {
              headers: { "User-Agent": USER_AGENT },
            });
            if (res.ok) {
              const sitemapText = await res.text();
              const $ = cheerio.load(sitemapText, { xmlMode: true });
              $("url > loc").each((_, el) => urls.add($(el).text().trim()));
            }
          } catch {}
        }
      }
    }
  } catch {}

  return urls;
}

async function main() {
  const args = process.argv.slice(2);
  let targetUrl = args[0] || "https://www.turismodealmeria.org/";

  if (!targetUrl.startsWith("http")) {
    targetUrl = "https://" + targetUrl;
  }

  const origin = new URL(targetUrl).origin;
  const hostname = new URL(origin).hostname.replace(/\./g, "_");

  console.log("\n========================================");
  console.log("  SITEMAP CRAWLER v2.0");
  console.log("========================================");
  console.log(`  Sitio: ${origin}`);
  console.log(`  Concurrencia: ${MAX_CONCURRENCY}`);
  console.log(`  Max paginas: ${MAX_PAGES}`);
  console.log("========================================");
  console.log("  Mejoras v2.0:");
  console.log("    - Prioridad inteligente por profundidad");
  console.log("    - Deteccion de canonical, noindex, hreflang");
  console.log("    - Excluye redirecciones, author, feeds, etc.");
  console.log("    - Extrae titulo, descripcion, H1, word count");
  console.log("    - Mide tiempo de respuesta");
  console.log("    - Genera informe JSON de auditoria");
  console.log("========================================\n");

  // Paso 1: Buscar sitemaps existentes
  const existingUrls = await tryExistingSitemap(origin);
  if (existingUrls.size > 0) {
    console.log(`  URLs de sitemap existente: ${existingUrls.size}`);
  }

  // Paso 2: Crawl
  console.log("\n  Iniciando rastreo...\n");
  queue.push(origin);
  for (const u of existingUrls) {
    if (isSameDomain(u, origin) && !shouldSkip(u)) {
      const normalized = normalizeUrl(u, origin);
      if (normalized && !queue.includes(normalized)) {
        queue.push(normalized);
      }
    }
  }

  const workers = [];
  for (let i = 0; i < MAX_CONCURRENCY; i++) {
    workers.push(worker(origin));
  }
  await Promise.all(workers);

  console.log(`\n\n  Rastreo completado.`);
  console.log(`  Total URLs rastreadas: ${visited.size}`);
  console.log(`  URLs fallidas: ${failed.size}`);

  if (visited.size === 0) {
    console.log("\n  No se encontraron URLs. Verifica que el sitio sea accesible.\n");
    process.exit(1);
  }

  // Generar sitemap
  const { xml, count, urls: sitemapUrls } = generateSitemap(origin);
  const sitemapFilename = `sitemap_${hostname}.xml`;
  const sitemapPath = path.join(process.cwd(), sitemapFilename);
  fs.writeFileSync(sitemapPath, xml, "utf-8");

  // Generar informe
  const report = generateReport(origin, sitemapUrls);
  const reportFilename = `report_${hostname}.json`;
  const reportPath = path.join(process.cwd(), reportFilename);
  fs.writeFileSync(reportPath, JSON.stringify(report, null, 2), "utf-8");

  // Resumen
  console.log("\n========================================");
  console.log("  RESULTADOS");
  console.log("========================================");
  console.log(`  URLs en sitemap: ${count}`);
  console.log(`  Redirecciones detectadas: ${report.redirects.length}`);
  console.log(`  Paginas noindex: ${report.noindex.length}`);
  console.log(`  Canonical mismatches: ${report.canonicalMismatches.length}`);
  console.log(`  Sin titulo: ${report.noTitle.length}`);
  console.log(`  Sin descripcion: ${report.noDescription.length}`);
  console.log(`  Sin H1: ${report.noH1.length}`);
  console.log(`  Contenido fino (<100 palabras): ${report.thinContent.length}`);
  console.log(`  Paginas lentas (>3s): ${report.slow.length}`);
  console.log(`  Tiempo respuesta medio: ${report.avgResponseTime}ms`);
  console.log("========================================");
  console.log(`  Sitemap: ${sitemapPath}`);
  console.log(`  Informe: ${reportPath}`);
  console.log("========================================\n");
}

main().catch((err) => {
  console.error("Error fatal:", err.message);
  process.exit(1);
});