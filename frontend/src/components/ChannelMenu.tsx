import '../App.css';

interface ChannelItem {
    title: string;
    icon: JSX.Element;
    link: string;
}

interface ChannelMenuProps {
    data: ChannelItem[];
    header: string;
}

const ChannelMenu: React.FC<ChannelMenuProps> = ({ data, header }) => {
    return (
        <div className='ChannelMenu'>
            <h1 className='header'>{header}</h1>
            <ul className="ChannelList">
            {data.map((val, key) => {
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