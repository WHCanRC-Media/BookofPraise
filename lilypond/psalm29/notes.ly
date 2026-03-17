melody = \relative c'' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a2 b2 a4 b4 cis2 d2 a2 r2 \break

  % Line 2
  d2 cis2 b4 d4 cis4 b4 a2 r2 \break

  % Line 3
  a2 fis4 b2 a4 g4 fis4 e2 r2 \break

  % Line 4
  fis2 g4 a2 g4 fis4 e4 d2 r2 \break

  % Line 5
  d2 a'2 a4 fis4 g4 a4 b2 a2 r2 \break

  % Line 6
  d2 a'2 a4 fis4 g4 a4 b2 a2 r2 \break

  % Line 7
  a2 b2 d4 cis4 b4 a4 g2 fis2 r2 \break

  % Line 8
  d2 fis4 a2 e4 g4 fis4 e2 d1 \bar "|."
}
