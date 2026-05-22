import i18next from 'i18next'
import { Alert } from '../UpdateHTML'
import { type SaveLoadError, SchemaValidationError } from './SaveErrors'

interface ReportOptions {
  // If true, also show a user-facing Alert with the error's i18n message.
  // Set to false for background paths (e.g. Steam Cloud sync) where the user did not initiate the action.
  alert?: boolean
}

export const logSaveError = (error: SaveLoadError): void => {
  if (error instanceof SchemaValidationError) {
    console.warn('[Save] schema validation failed:', error.toSafeJSON())
  } else if (error.cause !== undefined) {
    console.warn(`[Save] ${error.name}: ${error.message}`, { cause: error.cause })
  } else {
    console.warn(`[Save] ${error.name}: ${error.message}`)
  }
}

export const reportSaveError = async (
  error: SaveLoadError,
  options: ReportOptions = { alert: true }
): Promise<void> => {
  logSaveError(error)
  if (options.alert) {
    await Alert(i18next.t(error.userMessageKey))
  }
}
