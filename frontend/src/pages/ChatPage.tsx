import '../components/ChannelMenu';
import Chatbox from '../components/Chatbox';
import MessageDisplay from '../components/ChatDisplay';
import ChannelMenu from '../components/ChannelMenu';

interface ChatPageProps {
    messages: any[];
    sendMessage: (message: string) => void;
    messageDisplayRef: React.RefObject<HTMLDivElement>;
}

function ChatPage({ messages, sendMessage, messageDisplayRef }: ChatPageProps) {
    return (
        <div className='Chat'>
            <div className='ChannelMenu'>
                <ChannelMenu />
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