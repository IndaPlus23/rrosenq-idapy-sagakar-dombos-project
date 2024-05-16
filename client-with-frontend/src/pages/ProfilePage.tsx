interface ProfilePageProps{
  setUserName: React.Dispatch<React.SetStateAction<string | null>>;
  userName: string;
}

function ProfilePage({setUserName, userName}: ProfilePageProps) {

  const handleUserNameChange = () => {

    const newUserName = prompt("Enter new username");
    if (newUserName) {
      setUserName(newUserName);
      localStorage.setItem('userName', newUserName)
    }

  }
  
  return (
    <div className="ProfilePage">
      <div className="ProfilePic">
        <p>Profile Picture</p>
      </div>
      <div className="UserData">
        <p>Username: {userName}</p>
        <p>Full Name: </p>
        <p> Title: </p>
      </div>
      <div className="Changes">
        <h2>Profile Settings</h2>
        <p>Change Username</p>
        <button id="CU" onClick={handleUserNameChange}>Change Username</button>
        <p>Change Title</p>
        <textarea>New Title</textarea>
        <button id="CT">Change Title</button>
      </div>
    </div>
  )
}

export default ProfilePage
