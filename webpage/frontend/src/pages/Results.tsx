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

                <h2>Incognito</h2>
            </div>
        </div>
    )
}

export default Results;