\version "2.24.0"

\paper {
  line-width = 13\cm
  left-margin = 0\cm
  right-margin = 0\cm
}

\header {
  composer = "Geneva, 1562"
  tagline = ##f
}

melody = \relative c'' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  \once \hide Rest r4 b2 c2 b4 a4 g4 a4 f2 e2 r2 \break
  \omit Staff.Clef

  % Line 2
  \once \hide Rest r4 g2 a2 b4 g4 a4 c4 b2 a2 r2 \break

  % Line 3
  \once \hide Rest r4 a2 c2 b4 a4 a4 gis4 a2 r2 \break

  % Line 4
  \once \hide Rest r4 e2 f2 g4 a4 a4 gis4 a2 r2 \break

  % Line 5
  \once \hide Rest r4 a2 g2 a4 b4 c4 b4 a2 g2 r2 \break

  % Line 6
  \once \hide Rest r4 a2 g2 e4 f4 g4 e4 f2 e2 r2 \break

  % Line 7
  \once \hide Rest r4 e2 a2 g4 g4 a4 b4 c2 b2 r2 \break

  % Line 8
  \once \hide Rest r4 g2 a2 g4 e4 g4 g4 f2 e1 \once \hide Rest r2 \bar "|."
}


verse = \lyricmode {
  May a fu -- ture gen -- er -- a -- tion
praise the LORD for such sal -- va -- tion:
He looked down from hea -- ven high
to re -- lease those doomed to die!"
So in Zi -- on, in his dwell -- ing,
all will praise his love un -- fail -- ing
when the peo -- ples there a -- dore him
and the king -- doms kneel be -- fore him.
}


\score {
  <<
    \new Voice = "melody" { \melody }
    \new Lyrics \lyricsto "melody" { \verse }
  >>
  \layout {
    indent = 0
    \context {
      \Lyrics
      \override LyricText.self-alignment-X = #LEFT
    }
  }
}
