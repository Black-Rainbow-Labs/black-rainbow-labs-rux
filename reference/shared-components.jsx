// ===========================================================
// Storyboard page components — AI Studio
// ===========================================================

const SbIcon = ({ name, size = 18, stroke = 1.8 }) => {
  const paths = {
    search: <><circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/></>,
    plus: <><path d="M12 5v14M5 12h14"/></>,
    x: <><path d="M6 6l12 12M18 6L6 18"/></>,
    check: <><path d="M5 12l5 5L20 7"/></>,
    chevDown: <><path d="m6 9 6 6 6-6"/></>,
    chevLeft: <><path d="m15 6-6 6 6 6"/></>,
    chevRight: <><path d="m9 6 6 6-6 6"/></>,
    arrowRight: <><path d="M5 12h14M13 6l6 6-6 6"/></>,
    home: <><path d="M3 12 12 4l9 8"/><path d="M5 10v10h14V10"/></>,
    docs: <><path d="M8 3h8l4 4v14H4V3h4"/><path d="M9 3v4h6V3"/><path d="M8 12h8M8 16h5"/></>,
    spark: <><path d="M12 3v4M12 17v4M3 12h4M17 12h4M6 6l2.5 2.5M15.5 15.5 18 18M6 18l2.5-2.5M15.5 8.5 18 6"/></>,
    dots: <><circle cx="5" cy="12" r="1.2" fill="currentColor"/><circle cx="12" cy="12" r="1.2" fill="currentColor"/><circle cx="19" cy="12" r="1.2" fill="currentColor"/></>,
    film: <><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M3 9h18M3 15h18M8 4v16M16 4v16"/></>,
    storyboard: <><rect x="3" y="4" width="8" height="7" rx="1.5"/><rect x="13" y="4" width="8" height="7" rx="1.5"/><rect x="3" y="13" width="8" height="7" rx="1.5"/><rect x="13" y="13" width="8" height="7" rx="1.5"/></>,
    layers: <><path d="m12 3 9 5-9 5-9-5 9-5Z"/><path d="m3 13 9 5 9-5M3 18l9 5 9-5"/></>,
    folder: <><path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7Z"/></>,
    settings: <><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 0 1-4 0v-.1a1.7 1.7 0 0 0-1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 0 1 0-4h.1A1.7 1.7 0 0 0 4.6 9a1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 0 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9c.3.6.9 1 1.5 1H21a2 2 0 0 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1Z"/></>,
    bell: <><path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"/><path d="M10 21a2 2 0 0 0 4 0"/></>,
    user: <><circle cx="12" cy="8" r="4"/><path d="M4 21a8 8 0 0 1 16 0"/></>,
    trash: <><path d="M3 6h18M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/></>,
    download: <><path d="M12 3v12M7 10l5 5 5-5M4 21h16"/></>,
    heart: <><path d="M12 21s-8-5-8-11a5 5 0 0 1 9-3 5 5 0 0 1 9 3c0 6-8 11-8 11Z"/></>,
    star: <><path d="m12 3 2.8 6 6.2.8-4.5 4.4 1 6.1-5.5-3-5.5 3 1-6.1L3 9.8 9.2 9 12 3Z"/></>,
    chat: <><path d="M21 12a8 8 0 0 1-12 7l-5 1 1-5A8 8 0 1 1 21 12Z"/></>,
    cube: <><path d="m12 3 9 5v8l-9 5-9-5V8l9-5Z"/><path d="M3 8l9 5 9-5M12 13v10"/></>,
    image: <><rect x="3" y="3" width="18" height="18" rx="3"/><circle cx="9" cy="9" r="2"/><path d="m3 17 5-5 5 5 3-3 5 5"/></>,
    play: <><path d="M7 4v16l13-8L7 4Z" fill="currentColor" stroke="none"/></>,
    pause: <><rect x="6" y="4" width="4" height="16" rx="1" fill="currentColor" stroke="none"/><rect x="14" y="4" width="4" height="16" rx="1" fill="currentColor" stroke="none"/></>,
    music: <><path d="M9 18V6l12-2v12"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></>,
    imagePlus: <><rect x="3" y="3" width="14" height="14" rx="2"/><path d="M17 17h4M19 15v4"/><circle cx="8" cy="8" r="1.5"/><path d="m3 13 4-4 5 5"/></>,
    edit: <><path d="M16 4l4 4-11 11H5v-4L16 4Z"/></>,
    video: <><rect x="3" y="6" width="13" height="12" rx="2"/><path d="m22 8-6 4 6 4V8Z"/></>,
    upload: <><path d="M12 21V9M7 14l5-5 5 5M4 3h16"/></>,
    grip: <><circle cx="9" cy="6" r="1" fill="currentColor"/><circle cx="15" cy="6" r="1" fill="currentColor"/><circle cx="9" cy="12" r="1" fill="currentColor"/><circle cx="15" cy="12" r="1" fill="currentColor"/><circle cx="9" cy="18" r="1" fill="currentColor"/><circle cx="15" cy="18" r="1" fill="currentColor"/></>,
    expand: <><path d="M4 9V4h5M20 9V4h-5M4 15v5h5M20 15v5h-5"/></>,
    copy: <><rect x="8" y="8" width="13" height="13" rx="2"/><path d="M4 16V5a2 2 0 0 1 2-2h11"/></>,
    wand: <><path d="m15 4 2 2M19 8l2 2M9 15l-6 6M15 4l-6 6 4 4 6-6-4-4Z"/></>,
  };
  return (
    <svg viewBox="0 0 24 24" width={size} height={size} fill="none"
      stroke="currentColor" strokeWidth={stroke} strokeLinecap="round" strokeLinejoin="round">
      {paths[name]}
    </svg>
  );
};

// ---------- Project dropdown ----------
const ProjectDropdown = ({ projects, defaultIdx = 0 }) => {
  const [open, setOpen] = React.useState(false);
  const [idx, setIdx] = React.useState(defaultIdx);
  const ref = React.useRef(null);

  React.useEffect(() => {
    const h = (e) => { if (ref.current && !ref.current.contains(e.target)) setOpen(false); };
    document.addEventListener('mousedown', h);
    return () => document.removeEventListener('mousedown', h);
  }, []);

  return (
    <div className={`proj ${open ? 'is-open' : ''}`} ref={ref}>
      <button className="proj__trigger" onClick={() => setOpen(o => !o)}>
        <span className="proj__icon"><SbIcon name="folder" size={16}/></span>
        <div className="proj__body">
          <span className="proj__kicker">Project</span>
          <span className="proj__name">{projects[idx]}</span>
        </div>
        <span className="proj__chev"><SbIcon name="chevDown" size={15}/></span>
      </button>
      <div className="proj__menu">
        <div className="proj__menu-head">Recent projects</div>
        {projects.map((p, i) => (
          <div key={i}
            className={`proj__item ${i === idx ? 'is-selected' : ''}`}
            onClick={() => { setIdx(i); setOpen(false); }}>
            <span className="proj__item-dot"/>
            <span>{p}</span>
            {i === idx && <SbIcon name="check" size={14}/>}
          </div>
        ))}
        <div className="proj__menu-foot">
          <button className="proj__new"><SbIcon name="plus" size={14}/> New project</button>
        </div>
      </div>
    </div>
  );
};

// ---------- Top Nav ----------
const TopNav = ({ active = 'storyboard', onChange }) => {
  const items = [
    { id: 'home',       icon: 'home',       label: 'Home' },
    { id: 'library',    icon: 'layers',     label: 'Library' },
    { id: 'storyboard', icon: 'storyboard', label: 'Storyboard' },
    { id: 'studio',     icon: 'spark',      label: 'Studio' },
    { id: 'render',     icon: 'film',       label: 'Render' },
    { id: 'assets',     icon: 'image',      label: 'Assets' },
    { id: 'settings',   icon: 'settings',   label: 'Settings' },
  ];
  return (
    <nav className="topnav">
      {items.map(it => (
        <button key={it.id}
          className={`topnav__item ${active === it.id ? 'is-active' : ''}`}
          onClick={() => onChange && onChange(it.id)}
          title={it.label}>
          <SbIcon name={it.icon} size={20}/>
          <span>{it.label}</span>
        </button>
      ))}
    </nav>
  );
};

// ---------- Waveform (SVG, deterministic) ----------
const Waveform = ({ seed = 1, progress = 0.35, bars = 96 }) => {
  // deterministic pseudorandom
  const rand = (i) => {
    const x = Math.sin((i + 1) * seed * 12.9898) * 43758.5453;
    return x - Math.floor(x);
  };
  const pts = Array.from({ length: bars }, (_, i) => {
    const base = rand(i);
    const env = Math.sin((i / bars) * Math.PI); // fade at edges
    const mid = 0.35 + env * 0.6;
    return Math.max(0.08, base * mid);
  });
  const gradId = `wave-grad-${seed}`;
  return (
    <svg className="wave" viewBox={`0 0 ${bars * 3} 40`} preserveAspectRatio="none">
      <defs>
        <linearGradient id={gradId} x1="0" x2="1">
          <stop offset="0%" stopColor="#2F6BFF"/>
          <stop offset="55%" stopColor="#8B6BFF"/>
          <stop offset="100%" stopColor="#FF6B5B"/>
        </linearGradient>
      </defs>
      {pts.map((v, i) => {
        const h = v * 38;
        const played = (i / bars) < progress;
        return (
          <rect key={i}
            x={i * 3 + 0.5} y={20 - h/2}
            width={1.8} height={h} rx={0.9}
            fill={played ? `url(#${gradId})` : 'rgba(106,118,140,0.35)'}/>
        );
      })}
    </svg>
  );
};

// ---------- Scene Card ----------
const SceneCard = ({ scene, onDelete }) => {
  const [hasImage, setHasImage] = React.useState(scene.hasImage);
  const [hasAudio, setHasAudio] = React.useState(scene.hasAudio);
  const [playing, setPlaying] = React.useState(false);
  const [prompt, setPrompt] = React.useState(scene.prompt || '');
  const [duration, setDuration] = React.useState(scene.duration || 8);
  const [title, setTitle] = React.useState(scene.title);
  const [editingTitle, setEditingTitle] = React.useState(false);

  // Simulated playhead
  const [t, setT] = React.useState(0.28);
  React.useEffect(() => {
    if (!playing) return;
    const id = setInterval(() => {
      setT(p => {
        const n = p + 0.006;
        if (n > 1) { setPlaying(false); return 0; }
        return n;
      });
    }, 80);
    return () => clearInterval(id);
  }, [playing]);

  const mmss = (s) => `${Math.floor(s/60)}:${String(s%60).padStart(2,'0')}`;
  const startS = scene.startS ?? 0;
  const endS = startS + duration;

  return (
    <article className="scene">
      <header className="scene__head">
        {editingTitle ? (
          <input className="scene__title-input"
            value={title}
            autoFocus
            onBlur={() => setEditingTitle(false)}
            onKeyDown={(e) => e.key === 'Enter' && setEditingTitle(false)}
            onChange={e => setTitle(e.target.value)}/>
        ) : (
          <button className="scene__title" onClick={() => setEditingTitle(true)}>
            <span>{title}</span>
            <SbIcon name="edit" size={12}/>
          </button>
        )}
      </header>

      <button className={`scene__image ${hasImage ? 'has-image' : ''}`}
        onClick={() => setHasImage(v => !v)}>
        {hasImage && <div className="scene__image-preview" style={{ background: scene.imageBg }}/>}
      </button>

      <div className="scene__attach">
        <button className="attach" onClick={() => setHasAudio(v => !v)}>
          <SbIcon name="plus" size={11} stroke={2.4}/>
          <SbIcon name="music" size={13}/>
        </button>
        <button className="attach" onClick={() => setHasImage(v => !v)}>
          <SbIcon name="plus" size={11} stroke={2.4}/>
          <SbIcon name="image" size={13}/>
        </button>
      </div>

      <div className="scene__audio">
        <button className="wave-play" onClick={() => setPlaying(p => !p)}>
          <SbIcon name={playing ? 'pause' : 'play'} size={12}/>
        </button>
        <div className="wave-host">
          <Waveform seed={scene.id + 1} progress={hasAudio ? t : 0} bars={80}/>
        </div>
        <span className="wave-time">0:{String(Math.floor(t * duration)).padStart(2,'0')}</span>
      </div>

      <div className="scene__prompt-wrap">
        <textarea className="scene__prompt"
          value={prompt}
          onChange={e => setPrompt(e.target.value)}
          placeholder=""/>
      </div>

      <footer className="scene__foot">
        <div className="scene__duration">
          <span className="scene__foot-label">Duration</span>
          <div className="num-input">
            <input type="number" min={1} max={30} value={duration}
              onChange={e => setDuration(Number(e.target.value) || 1)}/>
            <div className="num-input__steps">
              <button onClick={() => setDuration(d => Math.min(30, d+1))}><SbIcon name="chevDown" size={10} stroke={2.4}/></button>
              <button onClick={() => setDuration(d => Math.max(1, d-1))}><SbIcon name="chevDown" size={10} stroke={2.4}/></button>
            </div>
          </div>
        </div>
        <div className="scene__time">
          <span>Start <b>{mmss(startS)}</b></span>
          <span>End <b>{mmss(endS)}</b></span>
        </div>
        <div className="scene__foot-actions">
          <button className="foot-btn" title="Generate video">
            <SbIcon name="video" size={14}/>
          </button>
          <button className="foot-btn" title="Delete scene" onClick={onDelete}>
            <SbIcon name="trash" size={14}/>
          </button>
        </div>
      </footer>
    </article>
  );
};

Object.assign(window, { SbIcon, ProjectDropdown, TopNav, Waveform, SceneCard });
