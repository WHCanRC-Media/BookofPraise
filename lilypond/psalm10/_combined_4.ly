\version "2.24.0"

\paper {
  line-width = 13\cm
  left-margin = 0\cm
  right-margin = 0\cm
}

\header {
  composer = "Geneva, 1542/1551"
  tagline = ##f
}

melody = \relative c'' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 a2 bes2 g2 f2 f2 e4 d4 f2 g2 a2 r2 \break
  \omit Staff.Clef

  % Line 2
  \once \hide Rest r4 a2 a4 b4 c4 b4 a4 g4 f2 e2 d2 r2 \break

  % Line 3
  \once \hide Rest r4 a2 bes2 g2 f2 f2 e4 d4 f2 g2 a2 r2 \break

  % Line 4
  \once \hide Rest r4 a2 a4 b4 c4 b4 a4 g4 f2 e2 d2 r2 \break

  % Line 5
  \once \hide Rest r4 d2 c4 b4 a4 g4 a4 b4 c2 b2 a2 r2 \break

  % Line 6
  \once \hide Rest r4 c2 b4 a4 g2 e2 f4 g4 a4 g4 f2 e2 r2 a2 c4 c4 d2 d2 c4 b4 a4 g4 f2 e2 d1 \once \hide Rest r2 \bar "|."
}


verse = \lyricmode {
  He sits in am -- bush watch -- ing for his prey
and mur -- ders those not of his wiles a -- ware.
He search -- es out the help -- less on their way.
He is a li -- on lur -- king in his lair.
He lies in wait to catch them in his snare.
Deep in his heart he thinks, "God does not see it;
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
