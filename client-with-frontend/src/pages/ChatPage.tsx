import '../components/ChannelMenu';
import Chatbox from '../components/Chatbox';
import MessageDisplay from '../components/ChatDisplay';
import ChannelMenu from '../components/ChannelMenu';
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
import Message from '../components/Message';
import { Event } from '@tauri-apps/api/event';

interface Channel {
    id: string;
    name: string;
    messages: string[];
}

interface ChatPageProps{
    messageDisplayRef: React.RefObject<HTMLDivElement>;
    userName: string;
}

function ChatPage({messageDisplayRef, userName}: ChatPageProps) {

    const [activeChannel, setActiveChannel] = useState<Channel | null>(null);
    const [channels, setChannels] = useState<Channel[]>([]);

    const switchChannel = (channelId: string) => {
        const channel = channels.find(c => c.id === channelId);
        if (channel) {
            setActiveChannel(channel);
        } else {
            const newChannel: Channel = {
                id: channelId,
                name: channelId,
                messages: [],
            };
            setActiveChannel(newChannel);
            setChannels(prevChannels => [...prevChannels, newChannel]);
        }
    };

    const sendMessage = (message: string) => {
        invoke('send_message', { message: message, target: activeChannel, visibility: 'public' })
    };

    useEffect(() => {
        if (messageDisplayRef.current) {
          messageDisplayRef.current.scrollTop = messageDisplayRef.current.scrollHeight;
        }
      }, [activeChannel ? activeChannel.messages : ''])
    
    // async function listen_messages() {
    //     await listen('recieve_message', (event: Event<Message>) => {
    //         let input = event.payload;
    //         let id = is_dm ? "dm-" + dm_name : "channel-" + input.channel;
    //         console.log(id);

    //         const scrollElem = document.getElementById(id);
    //         const chatElem = scrollElem.getElementsByClassName("inner-channel")[0];

    //         var para = document.createElement("p");
    //         para.innerHTML = '<strong class="who">' + input.username + ': </strong>' + input.body;
    //         chatElem.appendChild(para);

    //         scrollElem.scrollTop = scrollElem.scrollHeight;
    //     });
    // }
    
    return (
        <div className='Chat'>
            <div className='ChannelMenu'>
                <ChannelMenu 
                    header={"Channels"}
                    onChannelSelect={switchChannel}
                    activeChannelId={activeChannel ? activeChannel.id : ''}
                    />
            </div>
            <div ref={messageDisplayRef} className='ChatDisplay'>
                <MessageDisplay messages={activeChannel ? activeChannel.messages : []} />
            </div>
            <div className='Chatbox'>
                <Chatbox sendMessage={sendMessage} userName={userName} />
            </div>
        </div>
    );
}

export default ChatPage;
