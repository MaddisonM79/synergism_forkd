import type { ZodError, ZodIssue } from 'zod'

export class SaveLoadError extends Error {
  readonly userMessageKey: string

  constructor (userMessageKey: string, message: string, options?: ErrorOptions) {
    super(message, options)
    this.name = 'SaveLoadError'
    this.userMessageKey = userMessageKey
  }
}

export class SchemaValidationError extends SaveLoadError {
  readonly issues: ReadonlyArray<ZodIssue>

  constructor (issues: ReadonlyArray<ZodIssue>, userMessageKey = 'save.loadFailed') {
    const n = issues.length
    super(userMessageKey, `Save schema validation failed (${n} issue${n === 1 ? '' : 's'})`)
    this.name = 'SchemaValidationError'
    this.issues = issues
  }

  static fromZodError (error: ZodError, userMessageKey?: string): SchemaValidationError {
    return new SchemaValidationError(error.issues, userMessageKey)
  }

  // PII-safe issue summary. Excludes the rejected value — only path + code + zod's canonical message.
  toSafeJSON (): { path: string; code: string; message: string }[] {
    return this.issues.map((issue) => ({
      path: issue.path.join('.'),
      code: issue.code,
      message: issue.message
    }))
  }
}

export class SaveDecodeError extends SaveLoadError {
  constructor (cause: unknown, userMessageKey = 'importexport.unableImport') {
    super(userMessageKey, 'Save decode failed', { cause })
    this.name = 'SaveDecodeError'
  }
}
