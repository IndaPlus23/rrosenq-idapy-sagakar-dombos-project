import '../components/ChannelMenu';
import Chatbox from '../components/Chatbox';
import MessageDisplay from '../components/ChatDisplay';
import ChannelMenu from '../components/ChannelMenu';
import { ChannelData } from '../components/ChannelData';

interface ChatPageProps {
    messages: any[];
    sendMessage: (message: string) => void;
    messageDisplayRef: React.RefObject<HTMLDivElement>;
}

function ChatPage({ messages, sendMessage, messageDisplayRef }: ChatPageProps) {
    return (
        <div className='Chat'>
            <div className='ChannelMenu'>
                <ChannelMenu data={ChannelData} header={"Channels"}/>
            </div>
            <div ref={messageDisplayRef} className='ChatDisplay'>
                <MessageDisplay messages={messages} />
            </div>
            <div className='Chatbox'>
                <Chatbox sendMessage={sendMessage} />
            </div>
        </div>
    );
}

export default ChatPage;
