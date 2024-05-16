import { message } from "@tauri-apps/api/dialog";
import { invoke } from "@tauri-apps/api/tauri";

interface LoginPageProps {
  onLogin: (userName: string) => void;
}

function connectServer(ip: string, user: string, pass: string) {
  return new Promise<void>((resolve, reject) => {
    invoke('connect_server', { ip: ip, username: user, password: pass })
      .then(() => {
        resolve();
      })
      .catch((error: any) => {
        reject(error);
      });
  });
}

function doConnect() {
  const ipa = document.getElementById("ip-addr") as HTMLInputElement;
  const usr = document.getElementById("username") as HTMLInputElement;
  const passw = document.getElementById("password") as HTMLInputElement;
  return new Promise<void>((resolve, reject) => {
    connectServer(ipa.value, usr.value, passw.value).then(() => resolve()).catch(e => { console.error(e); message(e, { title: 'Tauri', type: 'error' }); reject(e) });
  })
}

function LoginPage({ onLogin }: LoginPageProps) {
  const handleLogin = () => {
    onLogin((document.getElementById("username") as HTMLInputElement).value);
  };

  return (
    <div className="LoginPage">

      <label htmlFor="ip-addr">IP:</label>
      <input type="text" id="ip-addr" defaultValue="127.0.0.1:5656" />

      <label htmlFor="username">Username:</label>
      <input type="text" id="username" defaultValue="rr" />

      <label htmlFor="password">Password:</label>
      <input type="password" id="password" defaultValue="123" />

      <button onClick={(e: any) => doConnect().then(handleLogin)}>Login</button>
    </div>
  );
}

export default LoginPage;
