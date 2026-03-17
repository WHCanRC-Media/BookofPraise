\version "2.24.0"

\paper {
  line-width = 13\cm
  left-margin = 0\cm
  right-margin = 0\cm
}

\header {
  composer = "Strasbourg, 1539 / Geneva, 1543"
  tagline = ##f
}

melody = \relative c'' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 a2 b4 cis4 d2 d2 cis4 b4 cis4 d4 b2 a2 r2 \break
  \omit Staff.Clef

  % Line 2
  \once \hide Rest r4 a2 a4 g4 fis2 fis2 fis4 e4 fis4 g4 e2 d2 r4 a'4 b2 d2 cis2 b2 cis4 d4 b4 b4 a2 r2 \break

  % Line 3
  \once \hide Rest r4 d2 d4 cis4 b2 a2 a4 b4 a4 g4 fis2 e2 r2 e2 a4 a4 b2 a2 a4 a4 b4 cis4 d2 cis2 \once \hide Rest r2 \break

  % Line 4
  r4 a4 b2 d2 cis2 b2 cis4 d4 b4 b4 a1 \once \hide Rest r2 \bar "|."
}


verse = \lyricmode {
  Life is like grass, so quick to fade and per -- ish,
The text "or like a flower that will but briefly flourish," only has 12 syllables, not 21. Here's the correct split:
that sears and withers in the blowing wind;
soon it is gone, not leav -- ing any tra -- ces.
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
