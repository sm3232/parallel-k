import { CartesianGrid, Legend, Line, LineChart, XAxis, YAxis } from 'recharts';
import '../styles/pages.css';

const Results = () => {
    return (
        <div className="page_container">
            <div className="top_section">
				<h1>Project Results</h1>

				<div className='top_section_background'></div>
			</div>

            <div className="page_sub_section">
                {/* Will find a component library for visuals */}
                {/* Probably just Recharts with some extra styles from Shadcn */}
                <h2>Mondrian</h2>

                <LineChart style={{ width: '100%', aspectRatio: 1.618, maxWidth: 800, margin: 'auto' }} responsive>
                    <CartesianGrid strokeDasharray="5 5" />
                    <XAxis dataKey="name" />
                    <YAxis width="auto" />
                    <Line type="monotone" dataKey="uv" />
                    <Line type="monotone" dataKey="pv" />
                    <Legend position="insideTopRight" offset={20} />
                </LineChart>

                <h2>Incognito</h2>
                <LineChart style={{ width: '100%', aspectRatio: 1.618, maxWidth: 800, margin: 'auto' }} responsive>
                    <CartesianGrid strokeDasharray="5 5" />
                    <XAxis dataKey="name" />
                    <YAxis width="auto" />
                    <Line type="monotone" dataKey="uv" />
                    <Line type="monotone" dataKey="pv" />
                    <Legend position="insideTopRight" offset={20} />
                </LineChart>
            </div>
        </div>
    )
}

export default Results;