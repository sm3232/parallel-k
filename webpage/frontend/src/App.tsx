import { useEffect } from 'react';
import { useTheme } from './stores/ThemesStore';
import { HashRouter } from 'react-router-dom';
import PageRouter from './pages/PageRouter';
import Navbar from './components/Navbar';
import Footer from './components/Footer';

import './App.css';

function App() {
	const theme = useTheme();

	useEffect(() => {
		document.documentElement.setAttribute('data-theme', theme);
	}, [theme]);

	return (
		<>
			<HashRouter>
				<div className="app_layout">
					<Navbar />
					<PageRouter />
					<Footer />
				</div>
			</HashRouter>
		</>
	)
}

export default App;