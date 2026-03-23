melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  e4 g4 b4 b4 b4 a4( c4 b4) a4 g2 \break

  % Line 2
  a4 b4 g4 e4 g4 a4( fis4 e4) d4 e2 \break

  % Line 3
  a4 a4 e4 e4 fis4 g2( fis4) e4 d2 \break

  % Line 4
  g4 a4 b4 b4 b4 a4( c4 b4) a4 g2 \break

  % Line 5
  d'4 d2 b4 b2 b4 a4( c4 b4) a4 g2 \break

  % Line 6
  a4 b4 g4 e4 g4 a4( fis4 e4) d4 e2 \bar "|."
}
