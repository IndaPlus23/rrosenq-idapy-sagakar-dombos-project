import { useState, useRef, useEffect } from 'react';
import './App.css';
import Navbar from './components/Navbar';
import Sidebar from "./components/Sidebar";
import { Routes, Route, Navigate } from 'react-router-dom';
import ChatPage from './pages/ChatPage';
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
              </div>
            } />
          </Routes>
        </div>
      </div>
    </div>
  );

}

export default App;