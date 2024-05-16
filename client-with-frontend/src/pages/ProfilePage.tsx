function ProfilePage() {
  return (
    <div className="ProfilePage">
      <div className="ProfilePic">
        <p>Profile Picture</p>
      </div>
      <div className="UserData">
        <p>Username: </p>
        <p>Full Name: </p>
        <p> Title: </p>
      </div>
      <div className="Changes">
        <h2>Profile Settings</h2>
        <p>Change Username</p>
        <textarea>New Username</textarea>
        <button id="CU">Change Username</button>
        <p>Change Title</p>
        <textarea>New Title</textarea>
        <button id="CT">Change Title</button>
      </div>
    </div>
  )
}

export default ProfilePage
