# Kensei Development Roadmap

This document tracks the strategic steps required to evolve the Kensei application into a full-scale web technology stack profiler.
This file is formatted to be synced automatically with GitHub Issues using the `xgh` roadmap standard.

## Infrastructure & Core Initialization <!-- phase:infrastructure -->

- [x] Dockerize frontend (Angular) and backend (FastAPI) environments (#1)
- [x] Integrate PostgreSQL and Redis architectures for persistence (#2)
- [x] Implement the Nothing Design System UI tokens and layout (#3)
- [x] Configure Docker-compose for rapid local development (HMR Support) (#4)

## Server Fingerprinting <!-- phase:server-fingerprint -->

- [x] Scaffold SQLAlchemy models for Profiles and Technologies (#5)
- [x] Create WebSocket endpoints for real-time profiling stream (#6)
- [x] Implement HTTP header analysis (Server, X-Powered-By, Set-Cookie) (#7)
- [x] Build SSL/TLS handshake profiler (certificate analysis, cipher suites) (#8)
- [x] Detect CDN, WAF, and reverse proxy layers (#9)

## JS Bundle Analysis <!-- phase:js-analysis -->

- [x] Implement JavaScript bundle download and parsing (#10)
- [x] Build dependency extraction from source maps (inline and external) (#11)
- [x] Create library fingerprint database with version detection (#12)
- [x] Detect build tools (webpack, vite, esbuild, rollup) and their versions (#13)
- [x] Analyze module federation and dynamic imports (#14)

## SPA Route Discovery <!-- phase:spa-routes -->

- [x] Implement Angular router pattern extraction (lazy-loaded modules, guards) (#15)
- [x] Detect React Router route definitions and nested layouts (#16)
- [x] Discover Vue Router configuration and named views (#17)
- [ ] Build client-side route map visualization (#18)
- [x] Detect authentication guards and protected routes (#19)

## Technology Inventory & Reporting <!-- phase:inventory -->

- [x] Build full stack composition report generator (#20)
- [x] Implement version mismatch and outdated dependency detection (#21)
- [x] Create JSON export for technology profiles (#22)
- [x] Add change detection over time (diff between scans) (#23)
- [ ] Build historical trend visualization (#24)

## Production Hardening <!-- phase:production-hardening -->

- [ ] Wrap FastAPI backend routes with JWT Authentication middleware (#25)
- [ ] Add RBAC to restrict profiling actions by user level (#26)
- [ ] Implement Redis-based rate limiting (#27)
- [ ] Setup scheduled recurrent profiling via Celery Beat tasks (#28)
