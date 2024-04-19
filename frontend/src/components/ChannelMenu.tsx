import '../App.css';
import { ChannelData } from './ChannelData';


function ChannelMenu() {
    return (
        <div className='ChannelMenu'>
            <h1 className='header'>Channels</h1>
            <ul className="ChannelList">
            {ChannelData.map((val, key) => {
                return (
                    <li 
                        key={key} 
                        className="row" 
                        id={window.location.pathname == val.link ? "active" : ""}
                        onClick={()=> {window.location.pathname=val.link}}>
                        <div id="icon">
                            {val.icon}
                        </div>   
                        <div id="title">
                            {val.title}
                        </div>
                    </li>
                )
            })}
            </ul>
        </div>
    )
}

export default ChannelMenu;