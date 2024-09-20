export * from './PeerReview'
export * from './ReaderWhitelist'
export * from './ResearchPaper'
export * from './ResearcherProfile'

import { ResearcherProfile } from './ResearcherProfile'
import { ResearchPaper } from './ResearchPaper'
import { PeerReview } from './PeerReview'
import { ReaderWhitelist } from './ReaderWhitelist'

export const accountProviders = {
  ResearcherProfile,
  ResearchPaper,
  PeerReview,
  ReaderWhitelist,
}
