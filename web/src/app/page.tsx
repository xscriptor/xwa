import ScanForm from "@/components/ScanForm";
import styles from "./page.module.css";

export default function Home() {
  return (
    <main className={styles.main}>
      <div className={styles.pageShell}>
        <div className={styles.hero} id="scanner">
          <div className={styles.heroContent}>
            <div className={styles.tagline}>SYSTEM_PROTOCOL: WEB_ANALYSIS_ENGINE</div>
            <h1 className={styles.title}>
              <span className={styles.titleAccent}>X</span>_WEB
              <span className={styles.titleAccent}>_ANALYZE</span>
            </h1>
            <p className={styles.subtitle}>
              Advanced reconnaissance. SEO integrity. Security posture. Accessibility compliance. Full-spectrum web analysis protocol.
            </p>
            <div className={styles.scanContainer} id="quick-start">
              <ScanForm />
            </div>
          </div>
        </div>
      </div>
    </main>
  );
}
