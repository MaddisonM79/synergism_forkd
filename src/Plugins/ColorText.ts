import type { Module } from 'i18next'

export default {
  type: 'postProcessor',
  name: 'ColorText',
  process: (value: string): string => {
    if (!value.includes('<<')) {
      return value
    }

    return value.replace(
      /<<(.*?)\|(.*?)>>/g,
      (_, color: string, text: string) => `<span style="color:${color.replace(/[<>"&]/g, '')}">${text}</span>`
    )
  }
} as Module
