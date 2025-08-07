'use server'

import { revalidatePath } from 'next/cache'

export async function reloadPage() {
  // Revalidate the current page to trigger a refresh
  revalidatePath('/users')
  // Return void as expected by form actions
}