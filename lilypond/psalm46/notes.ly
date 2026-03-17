melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 a'2 a4 a4 b4 g4 a4 c4 b2 a2 r2 \break

  % Line 2
  a2 a4 e4 g4 g4 fis4 d4 e2 d2 r2 \break

  % Line 3
  d2 a'4 b4 g4 a4( c2) b2 a2 r2 \break

  % Line 4
  e2 fis4 a4 e4 g4 fis2 e2 d2 r2 \break

  % Line 5
  a'2 a4 c4 b4 a4 b4 c4 d2 a2 r2 \break

  % Line 6
  a2 a4 c4 b4 a4 b4 c4 d2 a2 r2 \break

  % Line 7
  d2 d4 c4 b4 a4 g2 fis2 e2 \break

  % Line 8
  a2( c4) b4 a4 g4 fis2 e2 d1 \bar "|."
}
