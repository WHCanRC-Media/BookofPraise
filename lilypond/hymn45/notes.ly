melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d2 fis4 g4 a2 b4( cis4) d2 cis4( b4) a1 \break

  % Line 2
  a2 a4 a4 b2 a2 g2 fis2 e1 \break

  % Line 3
  fis2 fis4 e4 d4 fis4 a4( d4) b4( a4) g4( fis4) e1 \break

  % Line 4
  a2 b4 cis4 d2 g,4 fis2 e2 d1 \bar "|."
}
