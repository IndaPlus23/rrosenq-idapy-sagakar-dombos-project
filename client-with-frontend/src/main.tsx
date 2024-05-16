import React, { useState } from 'react';
import { createRoot } from 'react-dom/client';
import { BrowserRouter } from 'react-router-dom';
import App from './App';
import LoginPage from './LoginPage';

function Root() {

  const [isLoggedIn, setIsLoggedIn] = useState(sessionStorage.getItem('isLoggedIn'));

  const handleLogin = (newUserName: string) => {
    sessionStorage.setItem('userName', newUserName);
    sessionStorage.setItem('isLoggedIn', 'true');
    setIsLoggedIn('true');
  };

  return (
    <div>
      {isLoggedIn ? (
        <App />     
      ) : (
        <LoginPage onLogin={handleLogin} />
      )}
    </div>
  );
}

const root = createRoot(document.getElementById('root')!); // Create root
root.render( // Render your component inside root(
    <BrowserRouter>
      <Root />
    </BrowserRouter>
);
