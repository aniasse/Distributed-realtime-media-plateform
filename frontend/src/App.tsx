import { createBrowserRouter, RouterProvider } from 'react-router-dom'
import Lobby from './screens/Lobby'
import MeetingRoom from './screens/MeetingRoom'

const router = createBrowserRouter([
  {
    path: '/',
    element: <Lobby />,
    errorElement: <div>Erreur de chargement</div>,
  },
  {
    path: '/meeting/:roomId',
    element: <MeetingRoom />,
    errorElement: <div>Erreur de chargement de la salle</div>,
  },
])

function App() {
  return <RouterProvider router={router} />
}

export default App