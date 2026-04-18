import { useEffect } from 'react'
import { useQuery } from '@tanstack/react-query'
import WorkScreen from './components/WorkScreen'

function App() {
  const { data: me } = useQuery<{ username: string; repo_name?: string }>({
    queryKey: ['me'],
    queryFn: () => fetch('/api/me').then(r => r.json()),
  })

  useEffect(() => {
    if (me?.repo_name && me?.username) {
      document.title = `apm: ${me.repo_name}-${me.username}`
    }
  }, [me])

  return <WorkScreen />
}

export default App
