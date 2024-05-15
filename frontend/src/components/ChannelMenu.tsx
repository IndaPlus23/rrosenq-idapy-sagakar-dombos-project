import { useState } from 'react';
import '../App.css';
import AddIcon from '@mui/icons-material/Add';
import TagIcon from '@mui/icons-material/Tag';
import ArticleIcon from '@mui/icons-material/Article';

interface ChannelItem {
    title: string | null;
    icon: JSX.Element;
    link: string;
}

interface ChannelMenuProps {
    header: string;
    onChannelSelect: (channelId: string) => void;
    activeChannelId: string;
}

const ChannelMenu: React.FC<ChannelMenuProps> = ({ header, onChannelSelect, activeChannelId }) => {
    const [channels, setChannels] = useState<ChannelItem[]>([
        {
            title: "Main",
            icon: <TagIcon />,
            link: "/main",
        },

    ]);

    const handleNewChannelClick = () => {

        var input = prompt("Enter name")
        if (input) {
            const newChannel: ChannelItem = {
                title: input, // You can replace this with user input
                icon: <ArticleIcon />,
                link: "/new-channel" // You can create a link based on the new channel's title
            };
            setChannels([...channels, newChannel]);     
        }

    };

    return (
        <div className='ChannelMenu'>
            <h1 className='header'>{header}</h1>
            <ul className="ChannelList">
                {channels.map((val, key) => (
                    <li 
                        key={key} 
                        className="row" 
                        id={val.link === activeChannelId ? "active" : ""}
                        onClick={() => onChannelSelect(val.link)}>
                        <div id="icon">
                            {val.icon}
                        </div>   
                        <div id="title">
                            {val.title}
                        </div>
                    </li>
                ))}
            </ul>
            <div className='NewChannel'>
                <li onClick={handleNewChannelClick} style={{ cursor: 'pointer' }}>
                    <div id="ICON"><AddIcon/></div>
                    <div id='Text'>New Channel</div>
                </li>
            </div>
        </div>
    );
}

export default ChannelMenu;
