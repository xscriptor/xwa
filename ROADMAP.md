# XWA Development Roadmap

This document tracks the strategic steps required to evolve XWA into a full-scale modular web analysis ecosystem.
This file is formatted to be synced automatically with GitHub Issues using the `xgh` roadmap standard.

## Submodule Release Pipeline <!-- phase:releases -->

- [x] samurai v0.1.0 -- Web cybersecurity analysis platform (port scanning, web recon, DAST)
- [x] shinobi v0.1.0 -- Stealth web scraper with anti-blocking system
- [ ] kensei v0.1.0 -- Web technology stack profiler (JS bundle analysis, SPA route discovery, server fingerprinting)
- [ ] tengu v0.1.0 -- Web quality auditor (Core Web Vitals, WCAG compliance, structured data validation)
- [ ] kabuki v0.1.0 -- WAF and CDN analysis module (fingerprinting, challenge mechanism analysis, rate-limit profiling)
- [ ] yari v0.1.0 -- API security testing module (REST/GraphQL/gRPC endpoint discovery, fuzzing, auth testing)
- [ ] musha v0.1.0 -- Web content and DOM analysis module (structural diffing, third-party inventory, content drift)
- [ ] azuma v0.1.0 -- Web form and authentication flow analyzer (form discovery, OAuth mapping, session analysis)

## Cross-Module Integration <!-- phase:integration -->

- [ ] Define standard data exchange schemas across all submodules
- [ ] Angular shared component library consumed by all frontends
- [ ] Unified XWA docker-compose orchestration
- [ ] XWA API gateway for inter-module communication
- [ ] Common authentication and authorization layer
- [ ] Centralized result aggregation and correlation pipeline

## Infrastructure <!-- phase:infrastructure -->

- [ ] XWA monorepo git submodule initialization script
- [ ] CI/CD pipeline for cross-module integration testing
- [ ] Shared documentation site with per-module sections
- [ ] Common design system (Nothing Design) tokens and component library
- [ ] Release automation for coordinated version bumps

## Documentation <!-- phase:docs -->

- [ ] Architecture overview with module dependency graph
- [ ] Submodule contribution guide
- [ ] API contract specifications for cross-module communication
- [ ] Deployment guide for full XWA stack
