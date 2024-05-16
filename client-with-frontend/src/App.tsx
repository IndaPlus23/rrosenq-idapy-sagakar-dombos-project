import { useRef, useState, useEffect } from 'react';
import './App.css';
import Navbar from './components/Navbar';
import Sidebar from "./components/Sidebar";
import { Routes, Route, Navigate } from 'react-router-dom';
import ChatPage from './pages/ChatPage';
import ProfilePage from './pages/ProfilePage';
import SettingsPage from './pages/SettingsPage';
import DMPage from './pages/DMPage';
import ServerPage from './pages/ServerPage';

function App() {

  const messageDisplayRef = useRef<HTMLDivElement>(null);
  const [userName, setUserName] = useState<string | null>(null);

  useEffect(() => {
    const storedUserName = localStorage.getItem('userName');
    if (!storedUserName) {
      const userName = prompt("Enter your name");
      if (userName) {
        setUserName(userName);
        localStorage.setItem('userName', userName);
      }
    } else {
      setUserName(storedUserName);
    }
  }, []);

  return (
    <div className='App'>
      <div className='navbar'>
        <Navbar />
      </div>
      <div className='Components'>
        <div className='Sidebar'>
          <Sidebar />
        </div>
        <div className='PageContent'>
          <Routes>
            <Route path="" element={<Navigate to="/chat" />} />
            <Route path="/chat" element= {
              <div className='ChatPage'>
                <ChatPage
                  messageDisplayRef={messageDisplayRef}
                  userName={userName ? userName: ''}/>
              </div> } />
              <Route path="/dm" element= {
              <div className='DMPage'>
                <DMPage
                  messageDisplayRef={messageDisplayRef}
                  userName={userName ? userName: ''}
                />
              </div> } />
            <Route path="/profile" element= {
              <div className='ProfilePage'>
                <ProfilePage setUserName={setUserName} userName={userName ? userName: ''}/>
              </div> } />
            <Route path='/settings' element= {
              <div className='SettingsPage'>
                <SettingsPage />
              </div> } />
            <Route path="/changeServer" element= {
              <div className='ServerPage'>
                <ServerPage />
              </div> }/>
          </Routes>
        </div>
      </div>
    </div>
  );

}

export default App;