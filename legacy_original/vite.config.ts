import { defineConfig } from 'vite'
import { viteStaticCopy } from 'vite-plugin-static-copy'

// Vite handles the JS bundle, CSS, favicon, and index.html env substitution.
// Pictures/ and translations/ are dynamically referenced from TS at runtime
// (e.g. `Pictures/${set}/${icon}.png` template literals in src/Corruptions.ts,
// fetch(`./translations/${language}.json`) in src/i18n.ts), so they can't be
// statically analyzed and bundled — vite-plugin-static-copy copies them
// verbatim and serves them at /Pictures/ and /translations/ during dev.
//
// scripts/stage.mjs still renders _headers.template into build/_headers
// after the build — Vite doesn't model Cloudflare's _headers file natively.

export default defineConfig(({ mode }) => {
  return {
    publicDir: false,
    build: {
      outDir: 'build',
      emptyOutDir: true,
      sourcemap: mode !== 'production',
      target: 'es2020',
      // Disable base64 inlining of small assets — otherwise Vite inlines
      // ~1k Pictures references from index.html into the HTML, bloating it
      // from 395 KB to 2 MB. With inlining off, those references resolve to
      // hashed copies under /assets/. (TS code references Pictures via
      // template literals, which can't be statically analyzed, so the
      // vite-plugin-static-copy verbatim copies under /Pictures/ are still
      // needed. This is a known overlap to clean up in a later PR — likely
      // by moving Pictures/ + translations/ into a public/ subdir.)
      assetsInlineLimit: 0,
      rollupOptions: {
        input: 'index.html'
      }
    },
    server: {
      port: 3000,
      strictPort: true
    },
    preview: {
      port: 3000,
      strictPort: true
    },
    plugins: [
      viteStaticCopy({
        targets: [
          { src: 'Pictures', dest: '' },
          { src: 'translations', dest: '' },
          // MSW service worker must be served at the site root in dev so that
          // src/Synergism.ts's worker.start({ url: './mockServiceWorker.js' })
          // can register it. publicDir is disabled (see above), so we copy via
          // this plugin instead.
          { src: 'mockServiceWorker.js', dest: '' }
        ]
      })
    ]
  }
})
