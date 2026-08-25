import { useThemeToggle } from '../stores/ThemesStore';
import './styles/ThemeButton.css';

const ThemeButton = () => {
    const toggleTheme = useThemeToggle();

    return (
        <button className='theme_toggle_button' onClick={toggleTheme} />
    )
}

export default ThemeButton;