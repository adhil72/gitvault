import { createFileRoute } from '@tanstack/react-router'

import { WizardLayout } from '@/components/general/layouts/wizard-layout'

export const Route = createFileRoute('/_authed/_wizard')({ component: WizardLayout })
