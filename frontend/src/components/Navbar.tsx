import '../App.css'

const Navbar = () => {
    const handleClick = () => {
        console.log("hello");
    }

    return (
        <nav className="navbar">
            <h1 className="navbar-h1">My App</h1>
            <div className="links">
                <a href="#" className="navbar-link" onClick={handleClick}>Home</a>
                <a href="/create" className="navbar-link">New Blog</a>
            </div>
        </nav>
    );
}

export default Navbar;