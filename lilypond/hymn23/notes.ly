melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r4 d'4 b4 a4 g2 d4 e4 g4 a4 d,4 b'1 \break

  % Line 2
  a2 a4 g4 fis2 fis2 g4 a4 fis4 e4 d2 \break

  % Line 3
  d4 g2 g2 d'2 d4 c4 d4 c4( b4) a4 g4 a2 \break

  % Line 4
  d2( e4) d8( c8) d2 g,2 a8( b8 c4) b4 a2 g1 \bar "|."
}
