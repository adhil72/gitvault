import { createFileRoute } from '@tanstack/react-router'

import { SignInPage } from '@/components/pages/sign-in-page'

export const Route = createFileRoute('/_public/sign-in')({ component: SignInPage })
