import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import HomepageFeatures from '@site/src/components/HomepageFeatures';

import styles from './index.module.css';

function HomepageHeader() {
  const {siteConfig} = useDocusaurusContext();
  return (
    <header className={clsx('hero hero--primary', styles.heroBanner)}>
      <div className="container">
        <h1 className="hero__title">{siteConfig.title}</h1>
        <p className="hero__subtitle">{siteConfig.tagline}</p>
        <div className={styles.buttons}>
          <Link
            className="button button--secondary button--lg"
            to="/docs/getting-started">
            Get Started in 5 Minutes ⏱️
          </Link>
        </div>
        <div className={styles.quickStart}>
          <div className={styles.codeBlock}>
            <code>
              # Quick start<br/>
              git clone https://github.com/minifly/minifly.git<br/>
              cd minifly && cargo build --release<br/>
              ./target/release/minifly serve
            </code>
          </div>
        </div>
      </div>
    </header>
  );
}

export default function Home() {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout
      title={`${siteConfig.title} - Local Fly.io Development`}
      description="Local Fly.io development simulator with incredible DX. Test Machines API, LiteFS, and multi-region apps locally.">
      <HomepageHeader />
      <main>
        <HomepageFeatures />
        
        <section className={styles.showcase}>
          <div className="container">
            <div className="row">
              <div className="col col--12">
                <h2 className="text--center margin-bottom--lg">
                  Develop with Confidence
                </h2>
                <div className={styles.showcaseGrid}>
                  <div className={styles.showcaseItem}>
                    <h3>🎯 Perfect for Teams</h3>
                    <p>Standardize development environments across your team. No more "works on my machine" issues.</p>
                  </div>
                  <div className={styles.showcaseItem}>
                    <h3>⚡ Lightning Fast</h3>
                    <p>Hot reloading, instant deployments, and real-time log streaming. Built for speed.</p>
                  </div>
                  <div className={styles.showcaseItem}>
                    <h3>🔧 Production Parity</h3>
                    <p>Test exactly how your app will behave in production with full Fly.io API compatibility.</p>
                  </div>
                  <div className={styles.showcaseItem}>
                    <h3>📊 Rich Monitoring</h3>
                    <p>Structured logging, region context, and comprehensive status dashboards out of the box.</p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>
        
        <section className={styles.getStarted}>
          <div className="container">
            <div className="row">
              <div className="col col--8 col--offset-2">
                <div className="text--center">
                  <h2>Ready to Transform Your Development?</h2>
                  <p className="margin-bottom--lg">
                    Join developers who are already building better Fly.io apps with Minifly's 
                    local development environment.
                  </p>
                  <div className={styles.buttons}>
                    <Link
                      className="button button--primary button--lg margin-right--md"
                      to="/docs/getting-started">
                      Get Started
                    </Link>
                    <Link
                      className="button button--secondary button--lg"
                      to="/docs/examples/rust-axum">
                      View Examples
                    </Link>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>
      </main>
    </Layout>
  );
}