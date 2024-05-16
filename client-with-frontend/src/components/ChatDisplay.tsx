import Message from "./Message";

interface MessageDisplayProps {
  messages: Message[];
}

function MessageDisplay({ messages }: MessageDisplayProps) {
return (
  <div className="Box">
    {messages.map((message, index) => (
      <div key={index} className="Message">
          <p><strong>{message.username}</strong><small className="Time">{fromTimestamp(message.timestamp)}</small></p>
          <p>{message.body}</p>
      </div>
    ))}
  </div>
);
}

function fromTimestamp(timestamp: number): string{
  return new Date(timestamp * 1000).toLocaleString("sv-SE");
}

export default MessageDisplay;
