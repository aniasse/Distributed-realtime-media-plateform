import { useState } from 'react'
import { useNavigate } from 'react-router-dom'

const Lobby: React.FC = () => {
  const [roomId, setRoomId] = useState('123456')
  const [userName, setUserName] = useState('Aniasse')
  const navigate = useNavigate()

  const handleJoinRoom = () => {
    if (roomId && userName) {
      navigate(`/meeting/${roomId}?name=${userName}`)
    }
  }

  const handleCreateRoom = () => {
    // Generate new room ID
    const newRoomId = Math.random().toString(36).substr(2, 6).toUpperCase()
    setRoomId(newRoomId)
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100">
      <div className="max-w-md w-full p-6 bg-white rounded-2xl shadow-xl">
        <div className="text-center mb-8">
          <div className="text-4xl font-bold text-gray-900 mb-2">DRMP</div>
          <div className="text-sm text-gray-500">Plateforme de visioconférence distribuée</div>
        </div>

        <div className="space-y-4 mb-6">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Nom</label>
            <input
              type="text"
              value={userName}
              onChange={(e) => setUserName(e.target.value)}
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              placeholder="Votre nom"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">ID de la salle</label>
            <div className="relative">
              <input
                type="text"
                value={roomId}
                onChange={(e) => setRoomId(e.target.value.toUpperCase())}
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 pr-12"
                placeholder="Salle"
              />
              <button
                onClick={handleCreateRoom}
                className="absolute right-2 top-2 text-blue-600 hover:text-blue-800"
              >
                Générer
              </button>
            </div>
          </div>
        </div>

        <button
          onClick={handleJoinRoom}
          className="w-full bg-blue-600 text-white py-3 rounded-lg hover:bg-blue-700 transition-colors font-medium"
        >
          Rejoindre la salle
        </button>

        <div className="mt-6 text-center text-sm text-gray-500">
          <p>Version bêta - Plateforme DRMP</p>
        </div>
      </div>
    </div>
  )
}

export default Lobby