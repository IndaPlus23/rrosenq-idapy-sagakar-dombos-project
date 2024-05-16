import { ChangeEvent, FormEvent, useState } from 'react';

interface ChatboxProps {
    sendMessage: (message: string) => void;
    userName: string;
}

function Chatbox({ sendMessage, userName }: ChatboxProps) {
  const [message, setMessage] = useState('');

  const handleChange = (e: ChangeEvent<HTMLInputElement>) => {
    setMessage(e.target.value);
  };

  const handleSubmit = (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (message.trim() !== '') {
      sendMessage(message);
      setMessage('');
    }
  };

  return (
    <form onSubmit={handleSubmit}>
      <input className='Box'
        type="text"
        placeholder="Type a message..."
        value={message}
        onChange={handleChange}
      />
      <button type="submit" className='Button'>Send</button>
    </form>
  );
}

export default Chatbox;
