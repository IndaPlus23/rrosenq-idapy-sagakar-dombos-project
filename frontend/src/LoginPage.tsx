interface LoginPageProps {
  onLogin: (userName: string) => void;
}

function LoginPage({ onLogin }: LoginPageProps) {
  const handleLogin = () => {
    const newUserName = prompt("Enter name");
    if (newUserName) {
      onLogin(newUserName);
    }

  };

  return (
    <div className="LoginPage">
      <button onClick={handleLogin}>Login</button>
    </div>
  );
}

export default LoginPage;
