import '../App.css'

const Navbar = () => {

    return (
        <nav className="navbar">
            <h1 className="navbar-h1">Chatapplication</h1>
            <div className="links">
                <a href="/changeServer" className="navbar-link">Change Server</a>
            </div>
        </nav>
    );
}

export default Navbar;