import DOMPurify from 'dompurify'
import type { Module } from 'i18next'

// Only allow safe formatting tags inside translation strings, plus the <span>
// that this post-processor emits. The style attribute is restricted further
// below to color: only.
const ALLOWED_TAGS = ['span', 'b', 'i', 'em', 'strong', 'br', 'sup', 'sub']
const ALLOWED_ATTR = ['style']

// Translators (Crowdin contributors) are only partially trusted. The text slot
// of <<color|text>> ends up in innerHTML, so a malicious translator could ship
// <img src=x onerror=...> or arbitrary style="" payloads. Sanitize after
// substitution and constrain inline styles to `color:` only.
DOMPurify.addHook('uponSanitizeAttribute', (_node, data) => {
  if (data.attrName === 'style' && !/^color\s*:\s*[^;]+;?\s*$/i.test(data.attrValue)) {
    data.keepAttr = false
  }
})

export default {
  type: 'postProcessor',
  name: 'ColorText',
  process: (value: string): string => {
    if (!value.includes('<<')) {
      return value
    }

    const html = value.replace(
      /<<(.*?)\|(.*?)>>/g,
      (_, color: string, text: string) => `<span style="color:${color.replace(/[<>"&]/g, '')}">${text}</span>`
    )

    return DOMPurify.sanitize(html, { ALLOWED_TAGS, ALLOWED_ATTR })
  }
} as Module
