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
    messages: Message[];
}

interface ChatPageProps{
    messageDisplayRef: React.RefObject<HTMLDivElement>;
    userName: string;
}

function ChatPage({messageDisplayRef, userName}: ChatPageProps) {

    const [activeChannel, setActiveChannel] = useState<Channel | null>(null);
    const [channels, setChannels] = useState<Channel[]>([]);
    const [updated, setupdated] = useState<boolean>(false);
    const [firstChannel, setFirstChannel] = useState<boolean>(true)

    const switchChannel = (channelId: string) => {
        const channel = channels.find(c => c.id === channelId);
        if (channel) {
            setActiveChannel(channel); 
        }
    };

    const createChannel = (channelId: string) => {
        const newChannel: Channel = {
            id: channelId,
            name: channelId,
            messages: [],
        };
        channels.push(newChannel)
        if (firstChannel) {
            setActiveChannel(newChannel);
            setFirstChannel(false);
        }
    }
    
    useEffect(() => {
        async function listen_messages() {
            await listen('recieve_message', (event: Event<Message>) => {
                const channel = channels.find(c => c.id === event.payload.channel);
                if (channel) {
                    channel.messages.push(event.payload)
                    setupdated(true)
                }
            });
        }
        listen_messages()
    }, [])

    const sendMessage = (message: string) => {
        invoke('send_message', { message: message, target: activeChannel ? activeChannel.id : ""})
    };

    useEffect(() => {
        if (messageDisplayRef.current) {
          messageDisplayRef.current.scrollTop = messageDisplayRef.current.scrollHeight;
        }
        setupdated(false)
      }, [updated])
    
    
    return (
        <div className='Chat'>
            <div className='ChannelMenu'>
                <ChannelMenu 
                    header={"Channels"}
                    onChannelSelect={switchChannel}
                    activeChannelId={activeChannel ? activeChannel.id : ''}
                    isDM={false}
                    onChannelCreate={createChannel}
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
