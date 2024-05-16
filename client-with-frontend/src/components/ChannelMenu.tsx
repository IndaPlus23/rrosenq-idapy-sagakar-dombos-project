import { useState } from 'react';
import '../App.css';
import AddIcon from '@mui/icons-material/Add';
import TagIcon from '@mui/icons-material/Tag';
import ArticleIcon from '@mui/icons-material/Article';
import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { Event } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api';

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
    const [channels, setChannels] = useState<ChannelItem[]>([{
        title: "test",
        icon: <ArticleIcon />,
        link: "/test"
    }]);
    const addChannel = (name: string) => {

        const newChannel: ChannelItem = {
            title: name, // You can replace this with user input
            icon: <ArticleIcon />,
            link: "/" + name // You can create a link based on the new channel's title
        };
        setChannels([...channels, newChannel]);     

    };

    async function listen_channels() {
        await listen('init_channels', (event: Event<Array<string>>) => {
            for (const element of event.payload) {
                addChannel(element);
                invoke('request_history', { target: element, amount: '50', visibility: 'public' });
            }
        });
    }

    return (
        <div className='ChannelMenu' onLoad={listen_channels}>
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
        </div>
    );
}

export default ChannelMenu;
