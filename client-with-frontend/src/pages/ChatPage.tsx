import '../components/ChannelMenu';
import Chatbox from '../components/Chatbox';
import MessageDisplay from '../components/ChatDisplay';
import ChannelMenu from '../components/ChannelMenu';
import { useState, useEffect } from 'react';

interface Channel {
    id: string;
    name: string;
    messages: string[];
}

interface ChatPageProps{
    messageDisplayRef: React.RefObject<HTMLDivElement>;
}

function ChatPage({messageDisplayRef}: ChatPageProps) {

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
        if (activeChannel) {
            const updatedChannel = { ...activeChannel, messages: [...activeChannel.messages, message] };
            setActiveChannel(updatedChannel);
            setChannels(prevChannels => prevChannels.map(c => c.id === activeChannel.id ? updatedChannel : c));
        }
    };

    useEffect(() => {
        if (messageDisplayRef.current) {
          messageDisplayRef.current.scrollTop = messageDisplayRef.current.scrollHeight;
        }
      }, [activeChannel ? activeChannel.messages : ''])
    
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
                <Chatbox sendMessage={sendMessage} />
            </div>
        </div>
    );
}

export default ChatPage;
