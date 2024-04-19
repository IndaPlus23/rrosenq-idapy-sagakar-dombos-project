import HomeIcon from '@mui/icons-material/Home';
import ChatIcon from '@mui/icons-material/Chat';
import BubbleChartIcon from '@mui/icons-material/BubbleChart';
import SettingsIcon from '@mui/icons-material/Settings';
import PermIdentityIcon from '@mui/icons-material/PermIdentity';


export const SidebarData = [
    {
        title: "Home",
        icon: <HomeIcon />,
        link: "/home"
    },


    {
        title: "Chats",
        icon: <ChatIcon />,
        link: "/chats"
    },

    {
        title: "Direct message",
        icon: <BubbleChartIcon />,
        link: "/dm"
    },

    {
        title: "Settings",
        icon: <SettingsIcon />,
        link: "/settings"
    },

    {
        title: "Profile",
        icon: <PermIdentityIcon />,
        link: "/profile"
    },


]