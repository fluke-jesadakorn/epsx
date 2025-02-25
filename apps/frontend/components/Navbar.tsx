import {
  Menu,
  Switch,
  ConfigProvider,
  Flex,
  Button,
  Avatar,
  Tooltip,
} from "antd";
import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  HomeOutlined,
  LineChartOutlined,
  UserOutlined,
  MoonOutlined,
  SunOutlined,
  LogoutOutlined,
  LoginOutlined,
} from "@ant-design/icons";
import { useThemeStore } from "@/lib/store/theme";
import { useAuth } from "@/contexts/AuthContext";

const getMenuItems = (isLoggedIn: boolean) => [
  {
    label: <Link href="/">Home</Link>,
    key: "home",
    icon: <HomeOutlined />,
  },
  {
    label: <Link href="/ranking">Ranking</Link>,
    key: "ranking",
    icon: <LineChartOutlined />,
  },
  ...(isLoggedIn
    ? [
        {
          label: <Link href="/settings">Settings</Link>,
          key: "settings",
          icon: <UserOutlined />,
        },
      ]
    : []),
];

export default function Navbar() {
  const { theme, isDarkMode, toggleTheme } = useThemeStore();
  const pathname = usePathname();
  const { user, logout } = useAuth();
  const isLoggedIn = !!user?.firebaseUser;

  return (
    <ConfigProvider theme={theme}>
      <Flex align="center" justify="space-between" content="between">
        <Menu
          mode="horizontal"
          items={getMenuItems(isLoggedIn)}
          selectedKeys={[pathname === "/" ? "home" : pathname.split("/")[1]]}
          style={{
            justifyContent: "center",
            borderBottom: "none",
            padding: "0 24px",
            background: "transparent",
          }}
        />
        <Flex gap={8}>
          <Switch
            checked={isDarkMode}
            onChange={toggleTheme}
            checkedChildren={<MoonOutlined />}
            unCheckedChildren={<SunOutlined />}
            style={{ marginTop: 6 }}
          />
          {isLoggedIn && (
            <Tooltip title={user?.firebaseUser?.email}>
              <Link href="/settings">
                <Avatar
                  size="small"
                  icon={<UserOutlined />}
                  style={{ marginTop: 6 }}
                />
              </Link>
            </Tooltip>
          )}
          <Link href={isLoggedIn ? "#" : "/login"}>
            <Button
              type="text"
              icon={isLoggedIn ? <LogoutOutlined /> : <LoginOutlined />}
              onClick={isLoggedIn ? logout : undefined}
            >
              {isLoggedIn ? "Logout" : "Login"}
            </Button>
          </Link>
        </Flex>
      </Flex>
    </ConfigProvider>
  );
}
