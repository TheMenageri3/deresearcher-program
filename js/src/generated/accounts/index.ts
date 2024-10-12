export * from './PeerReview'
export * from './ResearchPaper'
export * from './ResearchTokenAccount'
export * from './ResearcherProfile'

import { ResearcherProfile } from './ResearcherProfile'
import { ResearchPaper } from './ResearchPaper'
import { PeerReview } from './PeerReview'
import { ResearchTokenAccount } from './ResearchTokenAccount'

export const accountProviders = {
  ResearcherProfile,
  ResearchPaper,
  PeerReview,
  ResearchTokenAccount,
}
