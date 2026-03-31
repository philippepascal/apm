import { useQuery } from '@tanstack/react-query'

function App() {
  const { data, error } = useQuery({
    queryKey: ['tickets'],
    queryFn: () => fetch('/api/tickets').then(r => {
      if (!r.ok) throw new Error(`/api/tickets returned ${r.status}`)
      return r.json()
    }),
  })

  if (data) console.log('tickets', data)
  if (error) console.error('tickets error', error)

  return <></>
}

export default App
