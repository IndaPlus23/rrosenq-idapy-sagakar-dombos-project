import { useState, useRef, useEffect } from 'react';
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

  const [messages, setMessages] = useState<string[]>([]);
  const messageDisplayRef = useRef<HTMLDivElement>(null);
  
  const sendMessage = (message: string) => {
    setMessages([...messages, message]);
  };

  useEffect(() => {
    if (messageDisplayRef.current) {
      messageDisplayRef.current.scrollTop = messageDisplayRef.current.scrollHeight;
    }
  }, [messages])

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
            <Route path="" element={
              <div className='ChatPage'>
                <Navigate replace to="/chat" />
              </div> } />
            <Route path="/chat" element= {
              <div className='ChatPage'>
                <ChatPage
                  messages={messages}
                  sendMessage={sendMessage}
                  messageDisplayRef={messageDisplayRef}
                />
              </div> } />
              <Route path="/dm" element= {
              <div className='DMPage'>
                <DMPage
                  messages={messages}
                  sendMessage={sendMessage}
                  messageDisplayRef={messageDisplayRef}
                />
              </div> } />
            <Route path="/profile" element= {
              <div className='ProfilePage'>
                <ProfilePage />
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