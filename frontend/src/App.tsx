import { useState, useRef, useEffect } from 'react';
import './App.css';
import Chatbox from './components/Chatbox';
import Navbar from './components/Navbar';
import Sidebar from "./components/Sidebar";
import MessageDisplay from './components/ChatDisplay';
import ChannelMenu from './components/ChannelMenu';

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
        <div className='ChannelMenu'>
          <ChannelMenu />
        </div>
        <div className='Chat'>
          <div ref={messageDisplayRef} className='ChatDisplay'>
            <MessageDisplay messages={messages} />
          </div>
          <div className='Chatbox'>
            <Chatbox sendMessage={sendMessage} />
          </div>
        </div> 
      </div>
    </div>
  );

}

export default App;